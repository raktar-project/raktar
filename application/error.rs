use anyhow::anyhow;
use aws_smithy_http::result::{CreateUnhandledError, SdkError};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use semver::Version;
use serde_json::json;
use std::num::ParseIntError;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("package info for {0} does not exist")]
    NonExistentPackageInfo(String),
    #[error("crate details for {0} does not exist")]
    NonExistentCrate(String),
    #[error("version {version} for {crate_name} does not exist")]
    NonExistentCrateVersion {
        crate_name: String,
        version: Version,
    },
    #[error("version {version} for {crate_name} already exists")]
    DuplicateCrateVersion {
        crate_name: String,
        version: Version,
    },
    #[error("package info for {0} does not exist")]
    Unauthorized(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error("unexpected error")]
    Other(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let detail = &self.to_string();
        let status_code = match &self {
            AppError::NonExistentPackageInfo(_) => StatusCode::NOT_FOUND,
            AppError::NonExistentCrate(_) => StatusCode::NOT_FOUND,
            AppError::NonExistentCrateVersion { .. } => StatusCode::NOT_FOUND,
            AppError::DuplicateCrateVersion { .. } => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = json!({ "errors": [{ "detail": detail }] });
        (status_code, Json(payload)).into_response()
    }
}

impl<E: std::error::Error + Send + Sync + CreateUnhandledError + 'static> From<SdkError<E>>
    for AppError
{
    fn from(err: SdkError<E>) -> Self {
        let error_message = format!("{}", err.into_service_error());
        let error_type = "aws_sdk".to_string();
        error!(error_message, error_type, "unexpected error");
        Self::Other(error_message)
    }
}

impl From<serde_dynamo::Error> for AppError {
    fn from(err: serde_dynamo::Error) -> Self {
        let error_message = format!("{}", err);
        let error_type = "serde_dynamo".to_string();
        error!(error_message, error_type, "unexpected error");
        Self::Other(error_message)
    }
}

impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        let error_message = err.to_string();
        let error_type = "parse_int_error".to_string();
        error!(error_message, error_type, "unexpected error");
        Self::Other(error_message)
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn internal_error() -> AppError {
    anyhow!("Internal Server Error").into()
}
