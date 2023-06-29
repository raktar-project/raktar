"""Stack for Raktar."""
import aws_cdk.aws_s3 as s3
from aws_cdk import Environment as CdkEnvironment
from aws_cdk import Stack
from constructs import Construct

from infrastructure.api import WebApi
from infrastructure.database import Database
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

        database = Database(self, "Database")
        table = database.table
        bucket = self.create_s3_bucket()
        backend_function = RustFunction(
            self,
            "RaktarFunction",
            bin_name="raktar-handler",
            description="Lambda function for the Raktar HTTP backend.",
            environment_variables={
                "TABLE_NAME": table.table_name,
                "CRATES_BUCKET_NAME": bucket.bucket_name,
                "DOMAIN_NAME": settings.api_domain,
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
            pre_token_trigger_function=pre_token_function,
            sso_metadata_url=settings.sso_metadata_url,
            app_domain=settings.app_domain,
            cognito_domain_prefix=settings.cognito_domain_prefix,
        )
        table.grant_read_write_data(backend_function)
        table.grant_read_write_data(pre_token_function)
        bucket.grant_read_write(backend_function)

        WebApi(
            self,
            "Api",
            api_name="raktar-web",
            api_lambda=backend_function,
            settings=settings,
            user_pool=user_pool,
        )

    def create_s3_bucket(self) -> s3.Bucket:
        return s3.Bucket(self, "CratesBucket")
