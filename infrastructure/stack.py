"""Stack for Raktar."""
import aws_cdk.aws_ec2 as ec2
from aws_cdk import Environment as CdkEnvironment
from aws_cdk import Stack
from constructs import Construct

from infrastructure.api import WebApi
from infrastructure.rust_function import RustFunction


class RaktarStack(Stack):
    """Raktar stack."""

    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        cdk_env: CdkEnvironment,
    ) -> None:
        """Define the stack."""
        super().__init__(scope, construct_id, env=cdk_env)

        vpc_id = self.node.try_get_context("vpc_id")
        vpc = ec2.Vpc.from_vpc_attributes(self, "Vpc", vpc_id=vpc_id)
        backend_function = RustFunction(
            self,
            "RaktarFunction",
            bin_name="raktar-api",
            description="Lambda function for the Raktar HTTP backend.",
            vpc=vpc,
        )

        WebApi(
            self,
            "Api",
            api_name="raktar",
            api_lambda=backend_function.function,
        )
