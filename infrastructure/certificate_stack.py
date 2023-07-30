"""Stack for Raktar."""
import os

import aws_cdk.aws_certificatemanager as certificate_manager
import aws_cdk.aws_route53 as route53
from aws_cdk import Environment, Stack
from constructs import Construct

from infrastructure.settings import Settings


class HostedUICertificateStack(Stack):
    """Stack to create a certificate for Cognito in us-east-1."""

    def __init__(
        self, scope: Construct, construct_id: str, *, settings: Settings
    ) -> None:
        """Create the certificate."""
        account = os.environ["CDK_DEFAULT_ACCOUNT"]
        env = Environment(account=account, region="us-east-1")

        super().__init__(scope, construct_id, env=env, cross_region_references=True)

        hosted_zone = route53.HostedZone.from_lookup(
            self,
            "HostedZone",
            domain_name=settings.hosted_zone_domain_name,
        )
        validation = certificate_manager.CertificateValidation.from_dns(hosted_zone)
        self._certificate = certificate_manager.Certificate(
            self,
            "Certificate",
            domain_name=settings.cognito_domain,
            validation=validation,
        )

    @property
    def certificate(self) -> certificate_manager.Certificate:
        """The certificate for hosted UI."""
        return self._certificate
