use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::models::user::User;
use crate::repository::Repository;
use crate::storage::CrateStorage;

#[derive(Debug, Serialize)]
pub struct ListOwnersResponse {
    users: Vec<User>,
}

pub async fn list_owners<R: Repository, S: CrateStorage>(
    Path(crate_name): Path<String>,
    State(app_state): State<AppState<R, S>>,
) -> AppResult<Json<ListOwnersResponse>> {
    let users = app_state.repository.list_owners(&crate_name).await?;
    let response = ListOwnersResponse { users };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct AddOwnersBody {
    users: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AddOwnersResponse {
    ok: bool,
    msg: String,
}

pub async fn add_owners<R: Repository, S: CrateStorage>(
    Path(crate_name): Path<String>,
    State(app_state): State<AppState<R, S>>,
    Json(new_owners): Json<AddOwnersBody>,
) -> AppResult<Json<AddOwnersResponse>> {
    app_state
        .repository
        .put_owners(&crate_name, new_owners.users)
        .await?;

    let response = AddOwnersResponse {
        ok: true,
        msg: "the users were successfully added as owners".to_string(),
    };
    Ok(response.into())
}
