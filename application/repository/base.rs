use semver::Version;

use crate::error::AppResult;
use crate::models::index::PackageInfo;
use crate::models::user::User;

#[async_trait::async_trait]
pub trait Repository: Clone + Send + Sync + 'static {
    async fn get_package_info(&self, crate_name: &str) -> AppResult<String>;
    async fn store_package_info(
        &self,
        crate_name: &str,
        version: &Version,
        package_info: PackageInfo,
    ) -> AppResult<()>;
    async fn set_yanked(&self, crate_name: &str, version: &Version, yanked: bool) -> AppResult<()>;
    async fn list_owners(&self, crate_name: &str) -> AppResult<Vec<User>>;
    async fn put_owners(&self, crate_name: &str, user_ids: Vec<String>) -> AppResult<()>;
}
