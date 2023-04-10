"""A serverless web API for the repository."""
from aws_cdk.aws_lambda import Function
from constructs import Construct


class WebApi(Construct):
    """The web API."""

    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        *,
        api_name: str,
        api_lambda: Function,
    ):
        """Create the API."""
        super().__init__(scope, construct_id)
