use anyhow::Result;
use semver::Version;

#[async_trait::async_trait]
pub trait CrateStorage: Clone + Send + Sync + 'static {
    async fn store_crate(&self, crate_name: &str, version: Version, data: Vec<u8>) -> Result<()>;
}
