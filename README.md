# Raktar

`raktar` is an alternative registry for Rust leveraging serverless AWS components.
Its aim is to offer a solution you can deploy in your AWS environment with a
few simple commands using AWS CDK.

It supports the new [sparse registry](https://blog.rust-lang.org/2022/06/22/sparse-registry-testing.html)
protocol. `git` based registry support is not planned.

There is no plan to support any currently widespread workaround for authentication.
The hope is that by the time development on this project is complete, the
[registry-auth](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#registry-auth)
flag will be stabilised.

> **Warning**
>
> This application is work in progress. The key APIs work, allowing you to publish crates, query the index, download crates, yank and unyank versions.
> However, the backend is somewhat useless without the frontend application that's in early stages of development and has not been open-sourced yet.

# Pre-requisites

The core application is written in pure Rust, however `raktar` uses Python for the infrastructure code (using AWS CDK). Therefore, you will need both to compile and deploy the application.

## Application pre-requisites
The following tools are required to compile and run the application:
- Rust

## Infrastructure pre-requisites
The following tools are required to deploy the service:
- Python 3.9+
- [poetry 1.4+](https://python-poetry.org/)
- an AWS account
- an [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html) profile with the appropriate permissions for CDK deployments
- [CDK CLI](https://docs.aws.amazon.com/cdk/v2/guide/getting_started.html)
- [cargo-lambda](https://www.cargo-lambda.info/) to compile Rust for AWS Lambda

# Deployment

To install the Python dependencies, run `poetry install`.

To configure the service, update the `infrastructure/.env` file with your preferred settings. `raktar` requires you to have a public hosted zone for your custom domain where it can create the necessary A record for the service. Set the domain name and the hosted zone ID in the `.env` file.

Once the service is configured, run

```shell
poetry run cdk deploy --profile <AWS_PROFILE> --all
```

to deploy the service.

To add the alternate registry, modify the `config.toml` (either in your project, or globally in `~/.cargo/config.toml`):

```toml
[registries.my-registry]
index = "sparse+https://{domain}/"
```

> **Note**
>
> `raktar` only support sparse indexes, not the original `git` based indexes.
