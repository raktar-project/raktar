"""A serverless web API designed for FastAPI."""
from aws_cdk import CfnOutput
from aws_cdk.aws_apigatewayv2_alpha import (
    CorsHttpMethod,
    CorsPreflightOptions,
    HttpApi,
    HttpMethod,
)
from aws_cdk.aws_apigatewayv2_integrations_alpha import HttpLambdaIntegration
from aws_cdk.aws_lambda import Function
from constructs import Construct

ALLOWED_HEADERS = [
    "Authorization",
    "Val-Cuid",
    "Content-Type",
]
ALLOWED_METHODS = [
    CorsHttpMethod.DELETE,
    CorsHttpMethod.GET,
    CorsHttpMethod.OPTIONS,
    CorsHttpMethod.POST,
    CorsHttpMethod.PUT,
]


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

        http_api = self.build_http_api(api_name=api_name)
        self.setup_lambda_integration(http_api, api_lambda)

        CfnOutput(self, "ApiUrl", value=http_api.url)

    @staticmethod
    def setup_lambda_integration(
        http_api: HttpApi,
        api_function: Function,
    ) -> None:
        """Set up the handler for Mangum/FastAPI."""
        integration = HttpLambdaIntegration(
            "LambdaIntegration",
            handler=api_function,
        )
        http_api.add_routes(
            path="/{proxy+}",
            methods=[
                HttpMethod.GET,
                HttpMethod.POST,
                HttpMethod.PUT,
                HttpMethod.PATCH,
                HttpMethod.DELETE,
            ],
            integration=integration,
        )

    def build_http_api(self, api_name: str) -> HttpApi:
        """Build the HTTP API."""
        return HttpApi(
            self,
            "HttpApi",
            api_name=api_name,
            cors_preflight=CorsPreflightOptions(
                allow_methods=ALLOWED_METHODS,
                allow_headers=ALLOWED_HEADERS,
                allow_origins=["*"],
            ),
        )
