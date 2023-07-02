use std::str::FromStr;

use axum::extract::{Path, State};
use semver::Version;

use crate::error::AppResult;
use crate::router::AppState;

pub async fn download_crate(
    Path((crate_name, version)): Path<(String, String)>,
    State((_, storage)): State<AppState>,
) -> AppResult<Vec<u8>> {
    let vers = Version::from_str(&version).expect("version to be valid");
    storage.get_crate(&crate_name, vers).await
}
