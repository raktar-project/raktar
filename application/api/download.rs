use std::str::FromStr;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use semver::Version;

use crate::app_state::AppState;
use crate::storage::CrateStorage;

pub async fn download_crate<S: CrateStorage>(
    Path((crate_name, version)): Path<(String, String)>,
    State(app_state): State<AppState<S>>,
) -> (StatusCode, Vec<u8>) {
    let vers = Version::from_str(&version).expect("version to be valid");
    match app_state.storage.get_crate(&crate_name, vers).await {
        Ok(data) => (StatusCode::OK, data),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, vec![]),
    }
}
