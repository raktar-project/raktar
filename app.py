"""Top level entrypoint for CDK."""
import os

import aws_cdk as cdk

from infrastructure.stack import RaktarStack

app = cdk.App()
try:
    cdk_env = cdk.Environment(
        region=os.environ["CDK_DEFAULT_REGION"],
        account=os.environ["CDK_DEFAULT_ACCOUNT"],
    )
except KeyError as e:
    raise RuntimeError(
        "Could not find AWS credentials, please ensure you're logged in."
    ) from e

RaktarStack(
    app,
    "RaktarStack",
    cdk_env=cdk_env,
)

app.synth()
