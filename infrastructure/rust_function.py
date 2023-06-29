"""Lambda construct for Rust."""
import subprocess
from enum import Enum
from pathlib import Path
from typing import Dict, Optional

import aws_cdk.aws_ec2 as ec2
from aws_cdk.aws_iam import Role
from aws_cdk.aws_lambda import Architecture as LambdaArchitecture
from aws_cdk.aws_lambda import Code, Function, Runtime
from constructs import Construct

_LAMBDA_DIR = "target/lambda/cdk"


class Architecture(Enum):
    """Architectures supported by Rust Lambdas."""

    ARM_64 = 1
    X86_64 = 2

    def as_lambda_architecture(self) -> LambdaArchitecture:
        mapping = {
            self.ARM_64: LambdaArchitecture.ARM_64,
            self.X86_64: LambdaArchitecture.X86_64,
        }

        return mapping[self]


class RustFunction(Function):
    """Rust-based Lambda function."""

    def __init__(
        self,
        scope: Construct,
        construct_id: str,
        *,
        bin_name: str,
        description: str,
        environment_variables: Optional[Dict[str, str]] = None,
        role: Optional[Role] = None,
        architecture: Architecture = Architecture.ARM_64,
        vpc: Optional[ec2.Vpc] = None,
    ):
        """Create a Rust function for a binary package."""

        _compile_lambda(bin_name, architecture)
        code_path = Path(_LAMBDA_DIR) / bin_name / "bootstrap.zip"

        super().__init__(
            scope,
            construct_id,
            function_name=bin_name,
            description=description,
            handler="doesnt.matter",
            runtime=Runtime.PROVIDED_AL2,
            architecture=architecture.as_lambda_architecture(),
            code=Code.from_asset(code_path.as_posix()),
            environment=environment_variables,
            role=role,
            vpc=vpc,
        )


def _compile_lambda(bin_name: str, architecture: Architecture):
    arch = "--arm64" if architecture == Architecture.ARM_64 else "--x86-64"
    command = [
        "cargo",
        "lambda",
        "build",
        "--release",
        "--output-format",
        "zip",
        "--bin",
        bin_name,
        "--lambda-dir",
        _LAMBDA_DIR,
        arch,
    ]

    subprocess.run(command, capture_output=False, text=True, check=True)
