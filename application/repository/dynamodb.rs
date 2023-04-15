use anyhow::anyhow;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use semver::Version;
use tracing::error;

use crate::error::{AppError, AppResult};

pub async fn set_yanked(
    db_client: &Client,
    crate_name: &str,
    version: &Version,
    yanked: bool,
) -> AppResult<()> {
    db_client
        .update_item()
        .table_name(get_table_name())
        .key("pk", AttributeValue::S(crate_name.to_string()))
        .key("sk", AttributeValue::S(version.to_string()))
        .update_expression("SET yanked = :y")
        .condition_expression("attribute_exists(sk)")
        .expression_attribute_values(":y", AttributeValue::Bool(yanked))
        .send()
        .await
        .map_err(|err| match err.into_service_error() {
            UpdateItemError::ConditionalCheckFailedException(_) => {
                AppError::NonExistentCrateVersion {
                    crate_name: crate_name.to_string(),
                    version: version.clone(),
                }
            }
            _ => {
                // TODO: add more information for the failure
                error!("failed to yank package");
                anyhow!("internal server error").into()
            }
        })?;

    Ok(())
}

pub fn get_table_name() -> String {
    std::env::var("TABLE_NAME").unwrap()
}
