use anyhow::anyhow;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use axum::extract::{Path, State};
use serde_dynamo::aws_sdk_dynamodb_0_25::from_items;
use tracing::error;

use crate::app_state::AppState;
use crate::db::get_table_name;
use crate::error::{AppError, AppResult};
use crate::models::index::PackageInfo;
use crate::storage::CrateStorage;

pub async fn get_info_for_short_name_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    Path(crate_name): Path<String>,
) -> AppResult<String> {
    assert_eq!(1, crate_name.len());

    get_crate_info(&app_state.db_client, &crate_name).await
}

pub async fn get_info_for_three_letter_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    Path((first_letter, crate_name)): Path<(String, String)>,
) -> AppResult<String> {
    assert_eq!(Some(first_letter.as_ref()), crate_name.get(0..1));

    get_crate_info(&app_state.db_client, &crate_name).await
}

pub async fn get_info_for_long_name_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    Path((first_two, second_two, crate_name)): Path<(String, String, String)>,
) -> AppResult<String> {
    assert_eq!(Some(first_two.as_ref()), crate_name.get(0..2));
    assert_eq!(Some(second_two.as_ref()), crate_name.get(2..4));

    get_crate_info(&app_state.db_client, &crate_name).await
}

async fn get_crate_info(db_client: &Client, crate_name: &str) -> AppResult<String> {
    let result = db_client
        .query()
        .table_name(get_table_name())
        .key_condition_expression("pk = :pk")
        .expression_attribute_values(":pk", AttributeValue::S(crate_name.to_string()))
        .send()
        .await
        .map_err(|err| {
            let error = format!("{:?}", err.into_service_error());
            error!(error, crate_name, "failed to query package info");
            anyhow!("internal server error")
        })?;

    match result.items() {
        None => Err(AppError::NonExistentPackageInfo(crate_name.to_string())),
        Some(items) => {
            let infos = from_items::<PackageInfo>(items.to_vec()).map_err(|_| {
                error!(
                    crate_name,
                    "failed to parse DynamoDB package info items for crate"
                );
                anyhow!("internal server error")
            })?;
            Ok(infos
                .into_iter()
                .map(|info| serde_json::to_string(&info).unwrap())
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }
}
