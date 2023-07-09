mod krate;
mod token;
pub mod user;

use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::transact_write_items::TransactWriteItemsError;
use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use semver::Version;
use serde_dynamo::aws_sdk_dynamodb_0_27::to_item;
use serde_dynamo::from_item;
use std::collections::HashMap;

use tracing::{error, info};

use crate::error::{AppError, AppResult};
use crate::models::crate_summary::CrateSummary;
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::repository::dynamodb::krate::CRATES_PARTITION_KEY;
use crate::repository::Repository;

#[derive(Clone)]
pub struct DynamoDBRepository {
    db_client: Client,
    table_name: String,
}

impl DynamoDBRepository {
    pub fn new(db_client: Client, table_name: String) -> Self {
        Self {
            db_client,
            table_name,
        }
    }

    pub fn new_from_env(db_client: Client) -> Self {
        Self::new(
            db_client,
            std::env::var("TABLE_NAME").expect("TABLE_NAME to be set in environment"),
        )
    }

    fn get_package_key(crate_name: &str) -> AttributeValue {
        AttributeValue::S(format!("CRT#{}", crate_name))
    }

    fn get_package_version_key(version: &Version) -> AttributeValue {
        AttributeValue::S(format!("V#{}", version))
    }

    fn get_package_metadata_key(version: &Version) -> AttributeValue {
        AttributeValue::S(format!("META#{}", version))
    }

    fn get_crate_info_key(&self, crate_name: String) -> Option<HashMap<String, AttributeValue>> {
        let mut key = HashMap::new();
        key.insert(
            "pk".to_string(),
            AttributeValue::S(CRATES_PARTITION_KEY.to_string()),
        );
        key.insert("sk".to_string(), AttributeValue::S(crate_name));
        Some(key)
    }

    async fn get_crate_details(&self, crate_name: &str) -> AppResult<Option<CrateSummary>> {
        let res = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .set_key(self.get_crate_info_key(crate_name.to_string()))
            .send()
            .await?;

        let details = if let Some(item) = res.item {
            let crate_info: CrateSummary = from_item(item)?;

            Some(crate_info)
        } else {
            None
        };

        Ok(details)
    }

    async fn put_package_version_with_new_details(
        &self,
        crate_name: &str,
        version: &Version,
        package_info: PackageInfo,
        crate_details: CrateSummary,
        is_new: bool,
    ) -> AppResult<()> {
        let item = to_item(crate_details)?;
        // TODO: when it's not new, this should probably verify we're not overwriting a competing write
        let condition_expression = if is_new {
            Some("attribute_not_exists(sk)".to_string())
        } else {
            None
        };
        let put_item = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .item("pk", AttributeValue::S(CRATES_PARTITION_KEY.to_string()))
            .item("sk", AttributeValue::S(crate_name.to_string()))
            .set_condition_expression(condition_expression)
            .build();
        let put_details_item = TransactWriteItem::builder().put(put_item).build();

        // TODO: fix unwrap
        let pk = DynamoDBRepository::get_package_key(&package_info.name);
        let sk = DynamoDBRepository::get_package_version_key(&package_info.vers);
        let item = to_item(package_info)?;
        let put = Put::builder()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .item("pk", pk)
            .item("sk", sk)
            .build();
        let put_item = TransactWriteItem::builder().put(put).build();

        match self
            .db_client
            .transact_write_items()
            .transact_items(put_details_item)
            .transact_items(put_item)
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
            Err(e) => Err(match e.into_service_error() {
                TransactWriteItemsError::TransactionCanceledException(_) => {
                    // TODO: how should we handle this? retry? fail?
                    anyhow::anyhow!("write conflict on new crate").into()
                }
                _ => anyhow::anyhow!("unexpected error in persisting crate").into(),
            }),
        }
    }

    async fn put_package_version(
        &self,
        crate_name: &str,
        version: &Version,
        package_info: PackageInfo,
    ) -> AppResult<()> {
        let pk = DynamoDBRepository::get_package_key(&package_info.name);
        let sk = DynamoDBRepository::get_package_version_key(&package_info.vers);

        let item = to_item(package_info)?;
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

    async fn put_package_metadata(&self, metadata: Metadata) -> AppResult<()> {
        let pk = Self::get_package_key(&metadata.name);
        let sk = Self::get_package_metadata_key(&metadata.vers);
        let item = to_item(metadata)?;
        self.db_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .item("pk", pk)
            .item("sk", sk)
            .send()
            .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Repository for DynamoDBRepository {}
