use anyhow::{bail, Result};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use axum::extract::{Path, State};
use serde_dynamo::aws_sdk_dynamodb_0_25::from_items;

use crate::app_state::AppState;
use crate::db::get_table_name;
use crate::models::index::PackageInfo;
use crate::storage::CrateStorage;

pub async fn get_info_for_short_name_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    Path(crate_name): Path<String>,
) -> String {
    assert_eq!(1, crate_name.len());

    get_crate_info(&app_state.db_client, &crate_name)
        .await
        .unwrap()
}

pub async fn get_info_for_three_letter_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    Path((first_letter, crate_name)): Path<(String, String)>,
) -> String {
    assert_eq!(Some(first_letter.as_ref()), crate_name.get(0..1));

    get_crate_info(&app_state.db_client, &crate_name)
        .await
        .unwrap()
}

pub async fn get_info_for_long_name_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    Path((first_two, second_two, crate_name)): Path<(String, String, String)>,
) -> String {
    assert_eq!(Some(first_two.as_ref()), crate_name.get(0..2));
    assert_eq!(Some(second_two.as_ref()), crate_name.get(2..4));

    get_crate_info(&app_state.db_client, &crate_name)
        .await
        .unwrap()
}

async fn get_crate_info(db_client: &Client, crate_name: &str) -> Result<String> {
    let result = db_client
        .query()
        .table_name(get_table_name())
        .key_condition_expression("pk = :pk")
        .expression_attribute_values(":pk", AttributeValue::S(crate_name.to_string()))
        .send()
        .await?;

    match result.items() {
        None => bail!("crate not found"),
        Some(items) => {
            let infos = from_items::<PackageInfo>(items.to_vec())?;
            let info_strings: Vec<String> = infos
                .into_iter()
                .map(|info| serde_json::to_string(&info).unwrap())
                .collect();

            Ok(info_strings.join("\n"))
        }
    }
}
