use anyhow::anyhow;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use semver::Version;
use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};
use tracing::{error, info};

use crate::error::{AppError, AppResult};
use crate::models::index::PackageInfo;
use crate::repository::Repository;

#[derive(Clone)]
pub struct DynamoDBRepository {
    db_client: Client,
    table_name: String,
}

impl DynamoDBRepository {
    pub(crate) fn new(db_client: Client) -> Self {
        Self {
            db_client,
            table_name: std::env::var("TABLE_NAME").unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl Repository for DynamoDBRepository {
    async fn get_crate_info(&self, crate_name: &str) -> AppResult<String> {
        let result = self
            .db_client
            .query()
            .table_name(&self.table_name)
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

    async fn store_package_info(
        &self,
        crate_name: &str,
        version: &Version,
        package_info: PackageInfo,
    ) -> AppResult<()> {
        let pk = aws_sdk_dynamodb::types::AttributeValue::S(package_info.name.clone());
        let sk = aws_sdk_dynamodb::types::AttributeValue::S(package_info.vers.to_string());

        let item = to_item(package_info).unwrap();
        match self
            .db_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .item("pk", pk)
            .item("sk", sk)
            .condition_expression("attribute_not_exists(sk)")
            .send()
            .await
        {
            Ok(_) => {
                info!(
                    crate_name = crate_name,
                    version = version.to_string(),
                    "persisted package info"
                );
                Ok(())
            }
            Err(err) => {
                let err = match err.into_service_error() {
                    PutItemError::ConditionalCheckFailedException(_) => {
                        AppError::DuplicateCrateVersion {
                            crate_name: crate_name.to_string(),
                            version: version.clone(),
                        }
                    }
                    _ => {
                        error!("failed to store package info");
                        anyhow::anyhow!("unexpected error in persisting crate").into()
                    }
                };

                Err(err)
            }
        }
    }

    async fn set_yanked(&self, crate_name: &str, version: &Version, yanked: bool) -> AppResult<()> {
        self.db_client
            .update_item()
            .table_name(&self.table_name)
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
}
