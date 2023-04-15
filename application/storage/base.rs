use semver::Version;

use crate::error::AppResult;

#[async_trait::async_trait]
pub trait CrateStorage: Clone + Send + Sync + 'static {
    async fn store_crate(&self, crate_name: &str, version: Version, data: Vec<u8>)
        -> AppResult<()>;
    async fn get_crate(&self, crate_name: &str, version: Version) -> AppResult<Vec<u8>>;
}
