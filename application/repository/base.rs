use std::sync::Arc;

use semver::Version;

use crate::auth::AuthenticatedUser;
use crate::error::AppResult;
use crate::models::crate_summary::CrateSummary;
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::models::token::TokenItem;
use crate::models::user::{CognitoUserData, User, UserId};

#[async_trait::async_trait]
pub trait Repository {
    async fn get_package_info(&self, crate_name: &str) -> AppResult<String>;
    async fn store_package_info(
        &self,
        crate_name: &str,
        version: &Version,
        package_info: PackageInfo,
        metadata: Metadata,
        authenticated_user: &AuthenticatedUser,
    ) -> AppResult<()>;
    async fn set_yanked(&self, crate_name: &str, version: &Version, yanked: bool) -> AppResult<()>;
    async fn list_owners(&self, crate_name: &str) -> AppResult<Vec<User>>;
    async fn add_owners(&self, crate_name: &str, user_ids: Vec<String>) -> AppResult<()>;
    async fn get_crate_summary(&self, crate_name: &str) -> AppResult<CrateSummary>;
    async fn get_all_crate_details(
        &self,
        filter: Option<String>,
        limit: usize,
    ) -> AppResult<Vec<CrateSummary>>;
    async fn get_crate_metadata(&self, crate_name: &str, version: &Version) -> AppResult<Metadata>;
    async fn list_crate_versions(&self, crate_name: &str) -> AppResult<Vec<Version>>;
    async fn store_auth_token(
        &self,
        token: &[u8],
        name: String,
        user_id: u32,
    ) -> AppResult<TokenItem>;
    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()>;
    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<TokenItem>>;
    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<TokenItem>>;
    async fn update_or_create_user(&self, user_data: CognitoUserData) -> AppResult<User>;
    async fn get_user_by_id(&self, user_id: UserId) -> AppResult<Option<User>>;
    async fn get_users(&self) -> AppResult<Vec<User>>;
}

pub type DynRepository = Arc<dyn Repository + Send + Sync>;
