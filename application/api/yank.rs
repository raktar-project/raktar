use std::str::FromStr;

use axum::extract::{Path, State};
use axum::Json;
use semver::Version;
use serde::Serialize;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::repository::Repository;
use crate::storage::CrateStorage;

#[derive(Serialize)]
pub struct Response {
    ok: bool,
}

pub async fn yank<R: Repository, S: CrateStorage>(
    Path((crate_name, version)): Path<(String, String)>,
    State(app_state): State<AppState<R, S>>,
) -> AppResult<Json<Response>> {
    let vers = Version::from_str(&version).expect("version to be valid");

    app_state
        .repository
        .set_yanked(&crate_name, &vers, true)
        .await?;

    let response = Json(Response { ok: true });
    Ok(response)
}
