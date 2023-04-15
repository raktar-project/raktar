use std::str::FromStr;

use axum::extract::{Path, State};
use axum::Json;
use semver::Version;
use serde::Serialize;

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::repository::dynamodb::set_yanked;
use crate::storage::CrateStorage;

#[derive(Serialize)]
pub struct Response {
    ok: bool,
}

pub async fn unyank<S: CrateStorage>(
    Path((crate_name, version)): Path<(String, String)>,
    State(app_state): State<AppState<S>>,
) -> AppResult<Json<Response>> {
    let vers = Version::from_str(&version).expect("version to be valid");

    set_yanked(&app_state.db_client, &crate_name, &vers, false).await?;

    let response = Json(Response { ok: true });
    Ok(response)
}
