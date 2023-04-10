"""Stack for Raktar."""
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

        backend_function = RustFunction(
            self,
            "RaktarFunction",
            bin_name="raktar-api",
            description="Lambda function for the Raktar HTTP backend.",
        )

        WebApi(
            self,
            "Api",
            api_name="raktar",
            api_lambda=backend_function.function,
        )
