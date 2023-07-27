"""Hosted UI for Cognito."""
import pathlib

import aws_cdk.aws_iam as iam
import aws_cdk.custom_resources as cr
from aws_cdk import CustomResource
from aws_cdk.aws_cognito import UserPool, UserPoolClient
from aws_cdk.aws_lambda import Architecture, Code, Function, Runtime
from aws_cdk.aws_logs import RetentionDays
from aws_cdk.aws_s3_assets import Asset
from constructs import Construct

CODE_PATH = pathlib.Path(__file__).parent / "custom_resources"
ASSETS_PATH = pathlib.Path(__file__).parent / "hosted_ui_assets"


class HostedUI(Construct):
    """Hosted UI customisations for Cognito."""

    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        user_pool: UserPool,
        client: UserPoolClient,
    ) -> None:
        """Create the resources for hosted UI customisations."""
        super().__init__(scope, construct_id)

        logo = self._create_logo_asset()
        css = self._create_css_asset()
        custom_function = self._create_custom_function(css, logo, user_pool, client)
        provider = self._create_provider(custom_function)

        CustomResource(
            self,
            "CustomResource",
            service_token=provider.service_token,
            properties={"css": css.s3_object_key, "logo": logo.s3_object_key},
        )

    def _create_provider(self, custom_function: Function) -> cr.Provider:
        role = iam.Role(
            self,
            "CognitoUiProviderRole",
            managed_policies=[
                iam.ManagedPolicy.from_aws_managed_policy_name(
                    managed_policy_name="AWSLambdaExecute"
                )
            ],
            assumed_by=iam.ServicePrincipal(service="lambda.amazonaws.com"),
        )

        return cr.Provider(
            self,
            "CognitoUiProvider",
            on_event_handler=custom_function,
            log_retention=RetentionDays.ONE_WEEK,  # default is INFINITE
            role=role,
        )

    def _create_logo_asset(self) -> Asset:
        return Asset(
            self,
            "CognitoHostedUiLogo",
            path=(ASSETS_PATH / "logo.png").as_posix(),
        )

    def _create_css_asset(self) -> Asset:
        return Asset(
            self,
            "CognitoHostedUiCss",
            path=(ASSETS_PATH / "cognito.css").as_posix(),
        )

    def _create_custom_function(
        self, css: Asset, logo: Asset, user_pool: UserPool, client: UserPoolClient
    ) -> Function:
        custom_function = Function(
            self,
            "CognitoSetupUiEventHandler",
            environment={
                "ASSET_BUCKET": logo.s3_bucket_name,
                "IMAGE_FILE_KEY": logo.s3_object_key,
                "CSS_KEY": css.s3_object_key,
                "USER_POOL_ID": user_pool.user_pool_id,
                "CLIENT_ID": client.user_pool_client_id,
            },
            runtime=Runtime.PYTHON_3_9,
            architecture=Architecture.ARM_64,
            handler="hosted_ui.handler",
            code=Code.from_asset(CODE_PATH.as_posix()),
            dead_letter_queue_enabled=True,
        )

        logo.grant_read(custom_function)
        css.grant_read(custom_function)

        custom_function.add_to_role_policy(
            statement=iam.PolicyStatement(
                actions=[
                    "cognito-idp:SetUICustomization",
                ],
                effect=iam.Effect.ALLOW,
                resources=[user_pool.user_pool_arn],
            )
        )

        return custom_function
