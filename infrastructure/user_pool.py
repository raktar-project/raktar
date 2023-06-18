import aws_cdk.aws_cognito as cognito
from aws_cdk import CfnOutput
from aws_cdk.aws_lambda import Function
from constructs import Construct


class RaktarUserPool(Construct):
    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        *,
        pre_token_trigger_function: Function,
        sso_metadata_url: str,
        app_domain: str,
        cognito_domain_prefix: str,
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
            app_domain,
        )
        self._user_pool.add_domain(
            "CognitoDomain",
            cognito_domain=cognito.CognitoDomainOptions(
                domain_prefix=cognito_domain_prefix
            ),
        )
        domain = f"https://{cognito_domain_prefix}.auth.{self._user_pool.stack.region}.amazoncognito.com"
        CfnOutput(self, "CognitoDomainOutput", value=domain)

    @property
    def user_pool_id(self) -> str:
        return self._user_pool.user_pool_id

    @property
    def user_pool_client_id(self) -> str:
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
        )

    def _build_user_pool_client(
        self,
        user_pool: cognito.UserPool,
        sso_provider: cognito.UserPoolIdentityProviderSaml,
        app_domain: str,
    ) -> cognito.UserPoolClient:
        provider = cognito.UserPoolClientIdentityProvider.custom(
            sso_provider.provider_name
        )
        return cognito.UserPoolClient(
            self,
            "CognitoClient",
            user_pool=user_pool,
            supported_identity_providers=[provider],
            o_auth=cognito.OAuthSettings(
                flows=cognito.OAuthFlows(authorization_code_grant=True),
                scopes=[cognito.OAuthScope.OPENID],
                callback_urls=[f"https://{app_domain}/cb", "http://localhost/cb"],
                logout_urls=[f"https://{app_domain}/", "http://localhost"],
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
            name="sso-provider",
            user_pool=user_pool,
        )
