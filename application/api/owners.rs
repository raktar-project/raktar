use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::AppState;
use raktar::error::AppResult;
use raktar::models::user::User;

#[derive(Debug, Serialize)]
pub struct ListOwnersResponse {
    users: Vec<User>,
}

pub async fn list_owners(
    Path(crate_name): Path<String>,
    State((repository, _)): State<AppState>,
) -> AppResult<Json<ListOwnersResponse>> {
    let users = repository.list_owners(&crate_name).await?;
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

pub async fn add_owners(
    Path(crate_name): Path<String>,
    State((repository, _)): State<AppState>,
    Json(new_owners): Json<AddOwnersBody>,
) -> AppResult<Json<AddOwnersResponse>> {
    repository.add_owners(&crate_name, new_owners.users).await?;

    let response = AddOwnersResponse {
        ok: true,
        msg: "the users were successfully added as owners".to_string(),
    };
    Ok(response.into())
}
