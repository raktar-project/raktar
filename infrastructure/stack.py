"""Stack for Raktar."""
import aws_cdk.aws_dynamodb as dynamodb
import aws_cdk.aws_s3 as s3
from aws_cdk import Environment as CdkEnvironment
from aws_cdk import Stack
from constructs import Construct

from infrastructure.api import WebApi
from infrastructure.rust_function import RustFunction
from infrastructure.settings import Settings
from infrastructure.user_pool import RaktarUserPool


class RaktarStack(Stack):
    """Raktar stack."""

    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        cdk_env: CdkEnvironment,
        settings: Settings,
    ) -> None:
        """Define the stack."""
        super().__init__(scope, construct_id, env=cdk_env, cross_region_references=True)
        self.settings = settings

        table = self.create_database_table()
        bucket = self.create_s3_bucket()
        backend_function = RustFunction(
            self,
            "RaktarFunction",
            bin_name="raktar-handler",
            description="Lambda function for the Raktar HTTP backend.",
            environment_variables={
                "TABLE_NAME": table.table_name,
                "CRATES_BUCKET_NAME": bucket.bucket_name,
                "DOMAIN_NAME": settings.domain_name,
            },
        )
        pre_token_function = RustFunction(
            self,
            "RaktarPreTokenFunction",
            bin_name="raktar-pre-token-handler",
            description="Lambda function for the Raktar Cognito user pool.",
            environment_variables={
                "TABLE_NAME": table.table_name,
            },
        )
        user_pool = RaktarUserPool(
            self,
            "RaktarUserPool",
            pre_token_trigger_function=pre_token_function.function,
            sso_metadata_url=settings.sso_metadata_url,
        )
        table.grant_read_write_data(backend_function.function)
        bucket.grant_read_write(backend_function.function)

        WebApi(
            self,
            "Api",
            api_name="raktar-web",
            api_lambda=backend_function.function,
            settings=settings,
            user_pool=user_pool,
        )

    def create_database_table(self) -> dynamodb.Table:
        return dynamodb.Table(
            self,
            "RegistryTable",
            partition_key=dynamodb.Attribute(
                name="pk", type=dynamodb.AttributeType.STRING
            ),
            sort_key=dynamodb.Attribute(name="sk", type=dynamodb.AttributeType.STRING),
            billing_mode=dynamodb.BillingMode.PROVISIONED,
            read_capacity=5,
            write_capacity=1,
        )

    def create_s3_bucket(self) -> s3.Bucket:
        return s3.Bucket(self, "CratesBucket")
