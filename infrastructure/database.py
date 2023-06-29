"""Infrastructure code for DynamoDB."""
import aws_cdk.aws_dynamodb as dynamodb
from constructs import Construct


class Database(Construct):
    """The database for application data."""

    def __init__(self, scope: Construct, construct_id: str):
        """Create the DynamoDB infrastructure."""
        super().__init__(scope, construct_id)

        self._table = self.create_database_table()
        self._setup_user_data_gsi(self._table)

    @property
    def table(self) -> dynamodb.Table:
        """The DynamoDB table."""
        return self._table

    def create_database_table(self) -> dynamodb.Table:
        """Create the DynamoDB table that will host application data."""
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

    @staticmethod
    def _setup_user_data_gsi(table: dynamodb.Table) -> None:
        """Create user -> tokens mapping.

        This is so that we can list all tokens for the logged-in user.
        """
        pk = dynamodb.Attribute(name="user_id", type=dynamodb.AttributeType.NUMBER)
        sk = dynamodb.Attribute(name="pk", type=dynamodb.AttributeType.STRING)
        table.add_global_secondary_index(
            index_name="user_tokens",
            projection_type=dynamodb.ProjectionType.ALL,
            partition_key=pk,
            sort_key=sk,
        )
