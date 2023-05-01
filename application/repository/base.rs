use std::sync::Arc;

use semver::Version;

use crate::error::AppResult;
use crate::models::crate_details::CrateDetails;
use crate::models::index::PackageInfo;
use crate::models::token::TokenItem;
use crate::models::user::User;

#[async_trait::async_trait]
pub trait Repository {
    async fn get_package_info(&self, crate_name: &str) -> AppResult<String>;
    async fn store_package_info(
        &self,
        crate_name: &str,
        version: &Version,
        package_info: PackageInfo,
    ) -> AppResult<()>;
    async fn set_yanked(&self, crate_name: &str, version: &Version, yanked: bool) -> AppResult<()>;
    async fn list_owners(&self, crate_name: &str) -> AppResult<Vec<User>>;
    async fn add_owners(&self, crate_name: &str, user_ids: Vec<String>) -> AppResult<()>;
    async fn get_all_crate_details(&self) -> AppResult<Vec<CrateDetails>>;
    async fn store_auth_token(&self, token: &[u8], name: String, user_id: u32) -> AppResult<()>;
    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()>;
    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<TokenItem>>;
    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<TokenItem>>;
    async fn get_or_create_user(&self, user_id: &str) -> AppResult<User>;
}

pub type DynRepository = Arc<dyn Repository + Send + Sync>;
