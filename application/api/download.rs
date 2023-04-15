use std::str::FromStr;

use axum::extract::{Path, State};
use semver::Version;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::repository::Repository;
use crate::storage::CrateStorage;

pub async fn download_crate<R: Repository, S: CrateStorage>(
    Path((crate_name, version)): Path<(String, String)>,
    State(app_state): State<AppState<R, S>>,
) -> AppResult<Vec<u8>> {
    let vers = Version::from_str(&version).expect("version to be valid");
    app_state.storage.get_crate(&crate_name, vers).await
}
