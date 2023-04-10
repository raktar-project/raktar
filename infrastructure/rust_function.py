"""Rust-based Lambda stuff."""
import subprocess
from pathlib import Path
from typing import Dict, Optional

from aws_cdk.aws_iam import Role
from aws_cdk.aws_lambda import Architecture, Code, Function, Runtime
from constructs import Construct

_LAMBDA_DIR = "target/lambda/cdk"


class RustFunction(Construct):
    """Rust-based Lambda function wrapper."""

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
    ):
        """Create a Rust function for a binary package."""
        super().__init__(scope, f"{construct_id}Wrapper")

        _compile_lambda(bin_name, architecture)
        code_path = Path(_LAMBDA_DIR) / bin_name / "bootstrap.zip"

        self._function = Function(
            self,
            construct_id,
            function_name=bin_name,
            description=description,
            handler="doesnt.matter",
            runtime=Runtime.PROVIDED_AL2,
            architecture=Architecture.ARM_64,
            code=Code.from_asset(code_path.as_posix()),
            environment=environment_variables,
            role=role,
        )

    @property
    def function(self):
        """The wrapped Lambda function."""
        return self._function


def _compile_lambda(bin_name: str, architecture: Architecture):
    arch = "--arm64" if architecture.ARM_64 else "--x86-64"
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
