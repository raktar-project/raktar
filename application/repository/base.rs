mod token;
mod user;

use std::sync::Arc;

use semver::Version;

use crate::auth::AuthenticatedUser;
use crate::error::AppResult;
use crate::models::crate_summary::CrateSummary;
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::models::user::User;
pub use crate::repository::base::token::TokenRepository;
pub use crate::repository::base::user::UserRepository;

#[async_trait::async_trait]
pub trait Repository: UserRepository + TokenRepository {
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
    async fn get_crate_summary(&self, crate_name: &str) -> AppResult<Option<CrateSummary>>;
    async fn get_all_crate_details(
        &self,
        filter: Option<String>,
        limit: usize,
    ) -> AppResult<Vec<CrateSummary>>;
    async fn get_crate_metadata(
        &self,
        crate_name: &str,
        version: &Version,
    ) -> AppResult<Option<Metadata>>;
    async fn list_crate_versions(&self, crate_name: &str) -> AppResult<Vec<Version>>;
}

pub type DynRepository = Arc<dyn Repository + Send + Sync>;
