"""Cognito infrastructure."""
import aws_cdk.aws_certificatemanager as certificate_manager
import aws_cdk.aws_cognito as cognito
import aws_cdk.aws_route53 as route53
import aws_cdk.aws_route53_targets as route53_targets
from aws_cdk.aws_lambda import Function
from constructs import Construct

from infrastructure.hosted_ui import HostedUI
from infrastructure.settings import Settings


class RaktarUserPool(Construct):
    """The Cognito user pool that hosts users."""

    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        *,
        pre_token_trigger_function: Function,
        sso_metadata_url: str,
        settings: Settings,
        hosted_zone: route53.IHostedZone,
        hosted_ui_certificate: certificate_manager.Certificate,
    ) -> None:
        """Set up the user pool."""
        super().__init__(scope, construct_id)

        self._user_pool = self._build_user_pool(pre_token_trigger_function)
        self._sso_provider = self._build_identity_provider(
            self._user_pool, sso_metadata_url
        )
        self._user_pool_client = self._build_user_pool_client(
            self._user_pool,
            self._sso_provider,
            settings,
        )
        user_pool_domain = self._user_pool.add_domain(
            "CognitoDomain",
            custom_domain=cognito.CustomDomainOptions(
                certificate=hosted_ui_certificate,
                domain_name=settings.cognito_domain,
            ),
        )
        target = route53_targets.UserPoolDomainTarget(user_pool_domain)
        route53.ARecord(
            self,
            "CognitoRecord",
            zone=hosted_zone,
            record_name=settings.cognito_domain,
            target=route53.RecordTarget.from_alias(target),
        )
        HostedUI(
            self, "HostedUICustomisations", self._user_pool, self._user_pool_client
        )

    @property
    def user_pool_id(self) -> str:
        """The ID of the user pool."""
        return self._user_pool.user_pool_id

    @property
    def user_pool_client_id(self) -> str:
        """The ID of the application client."""
        return self._user_pool_client.user_pool_client_id

    def _build_user_pool(
        self, pre_token_generation_function: Function
    ) -> cognito.UserPool:
        triggers = cognito.UserPoolTriggers(
            pre_token_generation=pre_token_generation_function,
        )
        return cognito.UserPool(
            self,
            "UserPool",
            user_pool_name="raktar-users",
            self_sign_up_enabled=False,
            lambda_triggers=triggers,
            standard_attributes=cognito.StandardAttributes(
                given_name=cognito.StandardAttribute(
                    required=True,
                    mutable=True,
                ),
                family_name=cognito.StandardAttribute(
                    required=True,
                    mutable=True,
                ),
            ),
        )

    def _build_user_pool_client(
        self,
        user_pool: cognito.UserPool,
        sso_provider: cognito.UserPoolIdentityProviderSaml,
        settings: Settings,
    ) -> cognito.UserPoolClient:
        callback_urls = [f"https://{settings.app_domain}/cb"]
        logout_urls = [f"https://{settings.app_domain}"]
        if settings.dev:
            callback_urls.append("http://localhost:5173/cb")
            logout_urls.append("http://localhost:5173")

        cognito_provider = cognito.UserPoolClientIdentityProvider.COGNITO
        custom_provider = cognito.UserPoolClientIdentityProvider.custom(
            sso_provider.provider_name
        )

        return cognito.UserPoolClient(
            self,
            "CognitoClient",
            user_pool=user_pool,
            supported_identity_providers=[cognito_provider, custom_provider],
            o_auth=cognito.OAuthSettings(
                flows=cognito.OAuthFlows(authorization_code_grant=True),
                scopes=[cognito.OAuthScope.OPENID],
                callback_urls=callback_urls,
                logout_urls=logout_urls,
            ),
        )

    def _build_identity_provider(
        self,
        user_pool: cognito.UserPool,
        sso_metadata_url: str,
    ) -> cognito.UserPoolIdentityProviderSaml:
        return cognito.UserPoolIdentityProviderSaml(
            self,
            "SSOProvider",
            metadata=cognito.UserPoolIdentityProviderSamlMetadata.url(sso_metadata_url),
            name="SSO",
            user_pool=user_pool,
            attribute_mapping=cognito.AttributeMapping(
                given_name=cognito.ProviderAttribute.other("given_name"),
                family_name=cognito.ProviderAttribute.other("family_name"),
            ),
        )
