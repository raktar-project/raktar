"""Stack for Raktar."""
import aws_cdk.aws_certificatemanager as certificate_manager
import aws_cdk.aws_route53 as route53
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
        hosted_ui_certificate: certificate_manager.Certificate,
    ) -> None:
        """Define the stack."""
        super().__init__(scope, construct_id, env=cdk_env, cross_region_references=True)
        self.settings = settings

        hosted_zone = self.get_hosted_zone(settings)
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
            settings=settings,
            hosted_zone=hosted_zone,
            hosted_ui_certificate=hosted_ui_certificate,
        )
        table.grant_read_write_data(backend_function)
        table.grant_read_write_data(pre_token_function)
        bucket.grant_read_write(backend_function)

        WebApi(
            self,
            "Api",
            api_name="raktar-web",
            api_lambda=backend_function,
            user_pool=user_pool,
            hosted_zone=hosted_zone,
            settings=settings,
        )

    def create_s3_bucket(self) -> s3.Bucket:
        """Create the S3 bucket where crates data will be stored."""
        return s3.Bucket(self, "CratesBucket")

    def get_hosted_zone(self, settings: Settings) -> route53.IHostedZone:
        """Get the hosted zone where the record routing to the API will be created."""
        return route53.HostedZone.from_lookup(
            self,
            "HostedZone",
            domain_name=settings.hosted_zone_domain_name,
        )
