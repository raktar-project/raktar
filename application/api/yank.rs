use anyhow::anyhow;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use std::str::FromStr;

use axum::extract::{Path, State};
use axum::Json;
use semver::Version;
use serde::Serialize;
use tracing::error;

use crate::app_state::AppState;
use crate::db::get_table_name;
use crate::error::{AppError, AppResult};
use crate::storage::CrateStorage;

#[derive(Serialize)]
pub struct Response {
    ok: bool,
}

pub async fn yank<S: CrateStorage>(
    Path((crate_name, version)): Path<(String, String)>,
    State(app_state): State<AppState<S>>,
) -> AppResult<Json<Response>> {
    let vers = Version::from_str(&version).expect("version to be valid");

    app_state
        .db_client
        .update_item()
        .table_name(get_table_name())
        .key("pk", AttributeValue::S(crate_name.clone()))
        .key("sk", AttributeValue::S(version.clone()))
        .update_expression("SET yanked = :y")
        .condition_expression("attribute_exists(sk)")
        .expression_attribute_values(":y", AttributeValue::Bool(true))
        .send()
        .await
        .map_err(|err| match err.into_service_error() {
            UpdateItemError::ConditionalCheckFailedException(_) => {
                AppError::NonExistentCrateVersion {
                    crate_name,
                    version: vers,
                }
            }
            _ => {
                // TODO: add more information for the failure
                error!("failed to yank package");
                anyhow!("internal server error").into()
            }
        })?;

    let response = Json(Response { ok: true });
    Ok(response)
}
