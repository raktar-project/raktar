"""The web API for the registry."""
import aws_cdk.aws_certificatemanager as acm
import aws_cdk.aws_route53 as route53
import aws_cdk.aws_route53_targets as route53_targets
from aws_cdk import CfnOutput, Stack
from aws_cdk.aws_apigatewayv2_alpha import (
    CorsHttpMethod,
    CorsPreflightOptions,
    DomainMappingOptions,
    DomainName,
    HttpApi,
    HttpMethod,
)
from aws_cdk.aws_apigatewayv2_authorizers_alpha import HttpJwtAuthorizer
from aws_cdk.aws_apigatewayv2_integrations_alpha import HttpLambdaIntegration
from aws_cdk.aws_lambda import Function
from constructs import Construct

from infrastructure.settings import Settings
from infrastructure.user_pool import RaktarUserPool

ALLOWED_HEADERS = [
    "Authorization",
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
        user_pool: RaktarUserPool,
        settings: Settings,
    ):
        """Create the API."""
        super().__init__(scope, construct_id)

        hosted_zone = self.get_hosted_zone(settings)
        certificate = acm.Certificate(
            self,
            "APICertificate",
            domain_name=settings.api_domain,
            validation=acm.CertificateValidation.from_dns(hosted_zone),
        )

        authorizer = self.build_http_authorizer(
            pool_id=user_pool.user_pool_id,
            client_id=user_pool.user_pool_client_id,
        )

        custom_domain = self.create_custom_domain(settings.api_domain, certificate)
        http_api = self.build_http_api(api_name=api_name, custom_domain=custom_domain)
        self.setup_lambda_integration(http_api, api_lambda, authorizer=authorizer)

        target = route53_targets.ApiGatewayv2DomainProperties(
            regional_domain_name=custom_domain.regional_domain_name,
            regional_hosted_zone_id=custom_domain.regional_hosted_zone_id,
        )
        route53.ARecord(
            self,
            "AliasRecord",
            zone=hosted_zone,
            record_name=settings.api_domain,
            target=route53.RecordTarget.from_alias(target),
        )

        CfnOutput(self, "ApiUrl", value=f"https://{settings.api_domain}/")

    @staticmethod
    def setup_lambda_integration(
        http_api: HttpApi,
        api_function: Function,
        authorizer: HttpJwtAuthorizer,
    ) -> None:
        """Set up the handler for Axum."""
        integration = HttpLambdaIntegration(
            "LambdaIntegration",
            handler=api_function,
        )
        http_api.add_routes(
            path="/gql",
            methods=[
                HttpMethod.GET,
                HttpMethod.POST,
            ],
            integration=integration,
            authorizer=authorizer,
        )
        http_api.add_routes(
            path="/{proxy+}",
            methods=[
                HttpMethod.GET,
                HttpMethod.POST,
                HttpMethod.PUT,
                HttpMethod.DELETE,
            ],
            integration=integration,
        )

    def build_http_api(self, api_name: str, custom_domain: DomainName) -> HttpApi:
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
            disable_execute_api_endpoint=True,
            default_domain_mapping=DomainMappingOptions(domain_name=custom_domain),
        )

    def create_custom_domain(
        self,
        domain_name: str,
        certificate: acm.Certificate,
    ) -> DomainName:
        return DomainName(
            self, "APIDomainName", domain_name=domain_name, certificate=certificate
        )

    def get_hosted_zone(self, settings: Settings) -> route53.IHostedZone:
        return route53.HostedZone.from_lookup(
            self,
            "HostedZone",
            domain_name=settings.hosted_zone_domain_name,
        )

    def build_http_authorizer(self, pool_id: str, client_id: str) -> HttpJwtAuthorizer:
        issuer = f"https://cognito-idp.{Stack.of(self).region}.amazonaws.com/{pool_id}"

        return HttpJwtAuthorizer(
            "JwtAuthorizer",
            jwt_issuer=issuer,
            jwt_audience=[client_id],
            authorizer_name="CognitoAuthorizer",
        )
