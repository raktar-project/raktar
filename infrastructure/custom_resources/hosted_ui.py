"""Custom resource to update the hosted UI in Cognito.

Mostly taken from: https://python.plainenglish.io/configuring-cognitos-hosted-ui-with-a-custom-resource-in-cdk-python-9f2fde423f95
"""
import logging
import os

import boto3
from crhelper import CfnResource

logger = logging.getLogger(__name__)

helper = CfnResource(
    json_logging=False,
    log_level="DEBUG",
    boto_level="CRITICAL",
    sleep_on_delete=120,
    ssl_verify=None,
)
cognito = boto3.client("cognito-idp")
s3 = boto3.resource("s3")

image = None
css = None

try:
    css = s3.Object(os.environ["ASSET_BUCKET"], os.environ["CSS_KEY"]).get()
    image = s3.Object(os.environ["ASSET_BUCKET"], os.environ["IMAGE_FILE_KEY"]).get()
except Exception as e:
    helper.init_failure(e)


def set_ui_customizations():
    try:
        css_data = css["Body"].read().decode("utf-8")
        image_data = image["Body"].read()
        cognito.set_ui_customization(
            UserPoolId=os.environ["USER_POOL_ID"],
            ClientId=os.environ["CLIENT_ID"],
            CSS=css_data,
            ImageFile=image_data,
        )
        logger.info("Updated Cognito Hosted UI")
    except Exception as e:
        logger.exception(e)
        raise ValueError(
            "An error occurred when attempting to set the UI customizations for the user pool client. See the CloudWatch logs for details"
        )


@helper.create
def create(event, context):
    logger.info("Got Create")
    set_ui_customizations()
    return None


@helper.update
def update(event, context):
    logger.info("Got Update")
    set_ui_customizations()
    return None


@helper.delete
def delete(event, context):
    logger.info("Got Delete")


@helper.poll_create
def poll_create(event, context):
    logger.info("Got create poll")
    # Return a resource id or True to indicate that creation is complete.
    # If True is returned an id will be generated
    return True


def handler(event, context):
    helper(event, context)
