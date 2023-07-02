use async_trait::async_trait;
use semver::Version;
use std::collections::HashMap;
use tokio::sync::RwLock;

use raktar::error::AppResult;
use raktar::storage::CrateStorage;

#[derive(Debug, Default)]
pub struct MemoryStorage {
    data: RwLock<HashMap<(String, Version), Vec<u8>>>,
}

#[async_trait]
impl CrateStorage for MemoryStorage {
    async fn store_crate(
        &self,
        crate_name: &str,
        version: Version,
        data: Vec<u8>,
    ) -> AppResult<()> {
        let key = (crate_name.to_string(), version);
        let mut lock = self.data.write().await;
        lock.insert(key, data);

        Ok(())
    }

    async fn get_crate(&self, crate_name: &str, version: Version) -> AppResult<Vec<u8>> {
        let key = (crate_name.to_string(), version);
        let lock = self.data.read().await;
        let data = lock.get(&key).cloned().unwrap();

        Ok(data)
    }
}
