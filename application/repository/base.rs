use crate::error::AppResult;
use crate::models::index::PackageInfo;
use semver::Version;

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
}
