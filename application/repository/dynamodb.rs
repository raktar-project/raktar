use anyhow::anyhow;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::transact_write_items::TransactWriteItemsError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::{AttributeValue, Put, ReturnValue, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use semver::Version;
use serde::Deserialize;
use serde_dynamo::aws_sdk_dynamodb_0_27::{from_items, to_item};
use serde_dynamo::from_item;
use std::collections::HashMap;
use std::str::FromStr;

use tracing::{error, info};

use crate::auth::AuthenticatedUser;
use crate::error::{internal_error, AppError, AppResult};
use crate::models::crate_details::CrateDetails;
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::models::token::TokenItem;
use crate::models::user::User;
use crate::repository::Repository;

static CRATES_PARTITION_KEY: &str = "CRATES";

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

    async fn get_crate_details(&self, crate_name: &str) -> AppResult<Option<CrateDetails>> {
        let res = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .set_key(self.get_crate_info_key(crate_name.to_string()))
            .send()
            .await?;

        let details = if let Some(item) = res.item {
            let crate_info: CrateDetails = from_item(item)?;

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
        crate_details: CrateDetails,
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

    async fn create_next_user(&self, login: &str) -> AppResult<User> {
        let next_id = self.find_next_user_id().await?;
        info!("next available ID is {}", next_id);

        self.put_new_user(login, next_id).await
    }

    async fn put_new_user(&self, login: &str, user_id: u32) -> AppResult<User> {
        let put = Put::builder()
            .table_name(&self.table_name)
            .item("pk", AttributeValue::S("USERS".to_string()))
            .item("sk", AttributeValue::S(format!("LOGIN#{}", login)))
            .item("id", AttributeValue::N(user_id.to_string()))
            .build();
        let put_login_mapping_item = TransactWriteItem::builder().put(put).build();

        let user_id_sk = AttributeValue::S(format!("ID#{:06}", user_id));
        let put = Put::builder()
            .table_name(&self.table_name)
            .item("pk", AttributeValue::S("USERS".to_string()))
            .item("sk", user_id_sk)
            .item("id", AttributeValue::N(user_id.to_string()))
            .item("login", AttributeValue::S(login.to_string()))
            .build();
        let put_user_item = TransactWriteItem::builder().put(put).build();

        let user = self
            .db_client
            .transact_write_items()
            .transact_items(put_login_mapping_item)
            .transact_items(put_user_item)
            .send()
            .await
            .map(|_| User {
                login: login.to_string(),
                id: user_id,
                name: None,
            })?;

        Ok(user)
    }

    async fn find_next_user_id(&self) -> AppResult<u32> {
        let output = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("pk = :pk AND begins_with(sk, :prefix)")
            .expression_attribute_values(":pk", AttributeValue::S("USERS".to_string()))
            .expression_attribute_values(":prefix", AttributeValue::S("ID#".to_string()))
            .scan_index_forward(false)
            .send()
            .await?;

        // TODO: review this, it's not safe to silently swallow all these
        let current_id = output
            .items()
            .and_then(|items| items.iter().next())
            .and_then(|item| item.get("id"))
            .and_then(|attr| attr.as_n().ok())
            .and_then(|id_string| u32::from_str(id_string).ok())
            .unwrap_or(0);

        Ok(current_id + 1)
    }
}

#[async_trait::async_trait]
impl Repository for DynamoDBRepository {
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
                let crate_details = CrateDetails {
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
                    let crate_details = CrateDetails {
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
                        // TODO: map login properly
                        id,
                        login: "dummy".to_string(),
                        name: None,
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

    async fn get_crate_details(&self, crate_name: &str) -> AppResult<CrateDetails> {
        let result = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(CRATES_PARTITION_KEY.to_string()))
            .key("sk", AttributeValue::S(crate_name.to_string()))
            .send()
            .await?;

        let item = result.item().cloned().ok_or(internal_error())?;
        let crate_details = from_item(item)?;

        Ok(crate_details)
    }

    async fn get_all_crate_details(
        &self,
        filter: Option<String>,
        limit: usize,
    ) -> AppResult<Vec<CrateDetails>> {
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
        let crates = from_items::<CrateDetails>(items.to_vec())?;

        Ok(crates)
    }

    async fn get_crate_metadata(&self, crate_name: &str, version: &Version) -> AppResult<Metadata> {
        let result = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", DynamoDBRepository::get_package_key(crate_name))
            .key("sk", DynamoDBRepository::get_package_metadata_key(version))
            .send()
            .await?;

        let item = result.item().cloned().ok_or(internal_error())?;
        let metadata = from_item(item)?;

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

    async fn store_auth_token(
        &self,
        token: &[u8],
        name: String,
        user_id: u32,
    ) -> AppResult<TokenItem> {
        let token_item = TokenItem::new(token, name, user_id);
        let item = to_item(token_item.clone())?;
        self.db_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;

        Ok(token_item)
    }

    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()> {
        let tokens = self.list_auth_tokens(user_id).await?;
        if let Some(token_to_delete) = tokens.into_iter().find(|item| item.token_id == token_id) {
            self.db_client
                .delete_item()
                .table_name(&self.table_name)
                .key("pk", AttributeValue::S(token_to_delete.pk))
                .key("sk", AttributeValue::S(token_to_delete.sk))
                .send()
                .await?;
        }

        Ok(())
    }

    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<TokenItem>> {
        // TODO: this shouldn't return TokenItem
        // in fact, we shouldn't leak TokenItem at all, as it's a DynamoDB model
        let output = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .index_name("user_tokens")
            .key_condition_expression("user_id = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::N(user_id.to_string()))
            .send()
            .await?;

        let items = output.items().map(|items| items.to_vec()).unwrap_or(vec![]);
        let tokens = from_items(items)?;

        Ok(tokens)
    }

    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<TokenItem>> {
        let output = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(TokenItem::get_pk(token)))
            .key("sk", AttributeValue::S(TokenItem::get_sk()))
            .send()
            .await?;

        let token_item = if let Some(item) = output.item().cloned() {
            Some(from_item(item)?)
        } else {
            None
        };

        Ok(token_item)
    }

    async fn get_or_create_user(&self, login: &str) -> AppResult<User> {
        #[derive(Debug, serde::Deserialize)]
        struct LoginNameMapping {
            id: u32,
        }

        // TODO: should we store down the name?
        let output = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S("USERS".to_string()))
            .key("sk", AttributeValue::S(format!("LOGIN#{}", login)))
            .send()
            .await?;

        // TODO: this has a race condition where two processes can both think the user doesn't exist yet
        match output.item().cloned() {
            None => {
                info!("user not found, creating new user");
                self.create_next_user(login).await
            }
            Some(item) => {
                let mapping: LoginNameMapping = from_item(item)?;
                Ok(User {
                    id: mapping.id,
                    login: login.to_string(),
                    name: None,
                })
            }
        }
    }
}
