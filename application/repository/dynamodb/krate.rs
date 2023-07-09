use anyhow::anyhow;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use semver::Version;
use serde::Deserialize;
use serde_dynamo::aws_sdk_dynamodb_0_27::from_items;
use serde_dynamo::from_item;
use std::str::FromStr;
use tracing::error;

use crate::auth::AuthenticatedUser;
use crate::error::{AppError, AppResult};
use crate::models::crate_summary::CrateSummary;
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::models::user::User;
use crate::repository::base::CrateRepository;
use crate::repository::DynamoDBRepository;

pub static CRATES_PARTITION_KEY: &str = "CRATES";

#[async_trait::async_trait]
impl CrateRepository for DynamoDBRepository {
    async fn get_package_info(&self, crate_name: &str) -> AppResult<String> {
        let result = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk and begins_with(sk, :prefix)")
            .expression_attribute_values(":pk", DynamoDBRepository::get_package_key(crate_name))
            .expression_attribute_values(":prefix", AttributeValue::S("V#".to_string()))
            .send()
            .await?;

        match result.items() {
            None => Err(AppError::NonExistentPackageInfo(crate_name.to_string())),
            Some(items) => {
                let infos = from_items::<PackageInfo>(items.to_vec())?;
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
        metadata: Metadata,
        authenticated_user: &AuthenticatedUser,
    ) -> AppResult<()> {
        match self.get_crate_details(crate_name).await? {
            // this is a brand new crate
            None => {
                let crate_details = CrateSummary {
                    name: crate_name.to_string(),
                    owners: vec![authenticated_user.id],
                    max_version: package_info.vers.clone(),
                    description: metadata.description.clone().unwrap_or("".to_string()),
                };
                self.put_package_version_with_new_details(
                    crate_name,
                    version,
                    package_info,
                    crate_details,
                    true,
                )
                .await?;
            }
            // this is an update to an existing crate
            Some(old_crate_details) => {
                if !old_crate_details.owners.contains(&authenticated_user.id) {
                    return Err(AppError::Unauthorized(
                        "user is not an owner of this package".to_string(),
                    ));
                }

                // should we update the head state of the crate?
                // the head state represents the latest version, so while it's valid to
                // publish a non-head version, this should not affect the crate details
                if old_crate_details.max_version < package_info.vers {
                    let crate_details = CrateSummary {
                        name: crate_name.to_string(),
                        owners: old_crate_details.owners,
                        max_version: package_info.vers.clone(),
                        description: metadata.description.clone().unwrap_or("".to_string()),
                    };
                    self.put_package_version_with_new_details(
                        crate_name,
                        version,
                        package_info,
                        crate_details,
                        false,
                    )
                    .await?;
                } else {
                    self.put_package_version(crate_name, version, package_info)
                        .await?;
                }
            }
        }

        self.put_package_metadata(metadata).await
    }

    async fn set_yanked(&self, crate_name: &str, version: &Version, yanked: bool) -> AppResult<()> {
        let pk = DynamoDBRepository::get_package_key(crate_name);
        let sk = DynamoDBRepository::get_package_version_key(version);

        self.db_client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", pk)
            .key("sk", sk)
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

    async fn list_owners(&self, crate_name: &str) -> AppResult<Vec<User>> {
        match self.get_crate_details(crate_name).await? {
            None => Err(AppError::NonExistentPackageInfo(crate_name.to_string())),
            Some(crate_details) => {
                let users = crate_details
                    .owners
                    .into_iter()
                    .map(|id| User {
                        // TODO: this is all dummy logic apart from the ID, it needs to be fixed
                        id,
                        login: "dummy".to_string(),
                        given_name: "dummy".to_string(),
                        family_name: "dummy".to_string(),
                    })
                    .collect();
                Ok(users)
            }
        }
    }

    async fn add_owners(&self, crate_name: &str, user_ids: Vec<String>) -> AppResult<()> {
        self.db_client
            .update_item()
            .table_name(&self.table_name)
            .set_key(self.get_crate_info_key(crate_name.to_string()))
            .update_expression("ADD #owners = :new_owners")
            .expression_attribute_names("#owners", "owners".to_string())
            .expression_attribute_values(":new_owners", AttributeValue::Ss(user_ids))
            .return_values(ReturnValue::UpdatedOld)
            .send()
            .await?;

        Ok(())
    }

    async fn get_crate_summary(&self, crate_name: &str) -> AppResult<Option<CrateSummary>> {
        let result = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(CRATES_PARTITION_KEY.to_string()))
            .key("sk", AttributeValue::S(crate_name.to_string()))
            .send()
            .await?;

        let crate_summary = if let Some(item) = result.item().cloned() {
            from_item(item)?
        } else {
            None
        };

        Ok(crate_summary)
    }

    async fn get_all_crate_details(
        &self,
        filter: Option<String>,
        limit: usize,
    ) -> AppResult<Vec<CrateSummary>> {
        let query_builder = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .limit(limit as i32);

        let query_builder = if let Some(prefix) = filter {
            query_builder
                .key_condition_expression("pk = :pk AND begins_with(sk, :prefix)")
                .expression_attribute_values(
                    ":pk",
                    AttributeValue::S(CRATES_PARTITION_KEY.to_string()),
                )
                .expression_attribute_values(":prefix", AttributeValue::S(prefix))
        } else {
            query_builder
                .key_condition_expression("pk = :pk")
                .expression_attribute_values(
                    ":pk",
                    AttributeValue::S(CRATES_PARTITION_KEY.to_string()),
                )
        };

        let output = query_builder.send().await?;
        let items = output.items().unwrap_or(&[]);
        let crates = from_items::<CrateSummary>(items.to_vec())?;

        Ok(crates)
    }

    async fn get_crate_metadata(
        &self,
        crate_name: &str,
        version: &Version,
    ) -> AppResult<Option<Metadata>> {
        let result = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", DynamoDBRepository::get_package_key(crate_name))
            .key("sk", DynamoDBRepository::get_package_metadata_key(version))
            .send()
            .await?;

        let metadata = if let Some(item) = result.item().cloned() {
            from_item(item)?
        } else {
            None
        };

        Ok(metadata)
    }

    async fn list_crate_versions(&self, crate_name: &str) -> AppResult<Vec<Version>> {
        #[derive(Debug, Deserialize)]
        struct QueryItem {
            sk: String,
        }

        let output = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk AND begins_with(sk, :prefix)")
            .expression_attribute_values(":pk", DynamoDBRepository::get_package_key(crate_name))
            .expression_attribute_values(":prefix", AttributeValue::S("V#".to_string()))
            .projection_expression("sk")
            .send()
            .await?;

        Ok(match output.items() {
            None => vec![],
            // TODO: fix unwraps
            Some(items) => {
                let sort_keys: Vec<QueryItem> = from_items(items.to_vec())?;
                sort_keys
                    .into_iter()
                    .map(|item| item.sk.strip_prefix("V#").unwrap().to_string())
                    .map(|version_string| Version::from_str(&version_string).unwrap())
                    .collect()
            }
        })
    }
}
