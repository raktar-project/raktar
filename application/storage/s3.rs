use anyhow::Result;
use aws_sdk_s3::Client;
use semver::Version;

use crate::storage::CrateStorage;

#[derive(Clone)]
pub struct S3Storage {
    bucket: String,
    prefix: String,
    client: Client,
}

impl S3Storage {
    pub(crate) async fn new() -> Self {
        let aws_config = aws_config::from_env().load().await;
        let bucket =
            std::env::var("CRATES_BUCKET_NAME").expect("S3 bucket to be configured in env");
        Self {
            bucket,
            prefix: "crates".to_string(),
            client: Client::new(&aws_config),
        }
    }

    pub fn crate_key(&self, name: &str, version: Version) -> String {
        format!("{}/{}/{}-{}.crate", self.prefix, name, name, version)
    }
}

#[async_trait::async_trait]
impl CrateStorage for S3Storage {
    async fn store_crate(&self, crate_name: &str, version: Version, data: Vec<u8>) -> Result<()> {
        let key = self.crate_key(crate_name, version);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.into())
            .send()
            .await?;

        Ok(())
    }

    async fn get_crate(&self, crate_name: &str, version: Version) -> Result<Vec<u8>> {
        let key = self.crate_key(crate_name, version);
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = output.body.collect().await?;
        Ok(data.into_bytes().to_vec())
    }
}
