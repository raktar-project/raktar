use anyhow::anyhow;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use aws_sdk_dynamodb::Client;
use semver::Version;
use serde_dynamo::aws_sdk_dynamodb_0_25::{from_items, to_item};
use serde_dynamo::from_item;
use thiserror::__private::AsDynError;
use tracing::{error, info};

use crate::error::{internal_error, AppError, AppResult};
use crate::models::crate_info::CrateInfo;
use crate::models::index::PackageInfo;
use crate::models::user::User;
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

    fn get_package_key(crate_name: &str) -> AttributeValue {
        AttributeValue::S(format!("CRT#{}", crate_name))
    }

    fn get_package_version_key(version: &Version) -> AttributeValue {
        AttributeValue::S(format!("V#{}", version))
    }

    fn get_crate_info_key() -> AttributeValue {
        AttributeValue::S("INFO".to_string())
    }
}

#[async_trait::async_trait]
impl Repository for DynamoDBRepository {
    async fn get_package_info(&self, crate_name: &str) -> AppResult<String> {
        let result = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk")
            .expression_attribute_values(":pk", DynamoDBRepository::get_package_key(crate_name))
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
        let pk = DynamoDBRepository::get_package_key(&package_info.name);
        let sk = DynamoDBRepository::get_package_version_key(&package_info.vers);

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
        match self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", DynamoDBRepository::get_package_key(crate_name))
            .key("sk", DynamoDBRepository::get_crate_info_key())
            .send()
            .await
        {
            Ok(output) => match output.item {
                None => Err(AppError::NonExistentPackageInfo(crate_name.to_string())),
                Some(item) => {
                    let crate_info: CrateInfo = from_item(item).map_err(|_| {
                        error!(crate_name, "failed to parse crate info");
                        internal_error()
                    })?;
                    let users = crate_info
                        .owners
                        .into_iter()
                        .map(|id| User {
                            // TODO: support login and name
                            id,
                            login: "dummy_login".to_string(),
                            name: None,
                        })
                        .collect();
                    Ok(users)
                }
            },
            Err(e) => {
                let err = e.as_dyn_error();
                error!(err, crate_name, "unexpected error in getting crate data");
                Err(internal_error())
            }
        }
    }

    async fn put_owners(&self, crate_name: &str, user_ids: Vec<String>) -> AppResult<()> {
        // TODO: store down the ID instead of the login name
        // we'll map the ID to login name on the read side
        match self
            .db_client
            .update_item()
            .table_name(&self.table_name)
            .key("pk", DynamoDBRepository::get_package_key(crate_name))
            .key("sk", DynamoDBRepository::get_crate_info_key())
            .update_expression("ADD #owners = :new_owners")
            .expression_attribute_names("#owners", "owners".to_string())
            .expression_attribute_values(":new_owners", AttributeValue::Ss(user_ids))
            .return_values(ReturnValue::UpdatedOld)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(_err) => Err(anyhow::anyhow!("internal server error").into()),
        }
    }
}
