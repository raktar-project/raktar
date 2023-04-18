use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use semver::Version;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("package info for {0} does not exist")]
    NonExistentPackageInfo(String),
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
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let detail = &self.to_string();
        let status_code = match &self {
            AppError::NonExistentPackageInfo(_) => StatusCode::NOT_FOUND,
            AppError::NonExistentCrateVersion { .. } => StatusCode::NOT_FOUND,
            AppError::DuplicateCrateVersion { .. } => StatusCode::BAD_REQUEST,
            AppError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = json!({ "errors": [{ "detail": detail }] });
        (status_code, Json(payload)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn internal_error() -> AppError {
    anyhow!("Internal Server Error").into()
}
