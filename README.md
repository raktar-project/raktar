<div align="center">

# Raktar

[![CI](https://github.com/raktar-registry/raktar/actions/workflows/ci.yml/badge.svg)](https://github.com/raktar-registry/raktar/actions/workflows/ci.yml)

**A serverless alternative registry for Rust.**

</div>

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
> The Raktar registry is meant to be used in conjunction with the [Raktar app](https://github.com/raktar-registry/raktar-app/).
>
> At this stage, Raktar aims to provide a good experience for a very specific setup. It makes a lot of assumption, for example that you'll authenticate using
> AWS Cognito using AWS IAM Identity Center (AWS SSO) as the identity provider.

# Installation Guide

Please refer to the website for detailed steps to install Raktar into your AWS environment.
