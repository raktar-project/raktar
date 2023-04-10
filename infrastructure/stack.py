"""Stack for Raktar."""
import aws_cdk.aws_dynamodb as dynamodb
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

        table = self.create_database_table()
        backend_function = RustFunction(
            self,
            "RaktarFunction",
            bin_name="raktar-handler",
            description="Lambda function for the Raktar HTTP backend.",
            environment_variables={
                "TABLE_NAME": table.table_name,
            },
        )
        table.grant_read_write_data(backend_function.function)

        WebApi(
            self,
            "Api",
            api_name="raktar-web",
            api_lambda=backend_function.function,
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
