use anyhow::anyhow;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::Client;
use semver::Version;

use crate::error::{AppError, AppResult};
use crate::storage::CrateStorage;

#[derive(Clone)]
pub struct S3Storage {
    bucket: String,
    prefix: String,
    client: Client,
}

impl S3Storage {
    pub async fn new() -> Self {
        let aws_config = aws_config::from_env().load().await;
        let bucket =
            std::env::var("CRATES_BUCKET_NAME").expect("S3 bucket to be configured in env");
        Self {
            bucket,
            prefix: "crates".to_string(),
            client: Client::new(&aws_config),
        }
    }

    pub fn crate_key(&self, name: &str, version: &Version) -> String {
        format!("{}/{}/{}-{}.crate", self.prefix, name, name, version)
    }
}

#[async_trait::async_trait]
impl CrateStorage for S3Storage {
    async fn store_crate(
        &self,
        crate_name: &str,
        version: Version,
        data: Vec<u8>,
    ) -> AppResult<()> {
        let key = self.crate_key(crate_name, &version);
        match self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.into())
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow::anyhow!("unexpected error in storing crate").into()),
        }
    }

    async fn get_crate(&self, crate_name: &str, version: Version) -> AppResult<Vec<u8>> {
        let key = self.crate_key(crate_name, &version);
        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Err(err) => {
                let mapped_err = match err.into_service_error() {
                    GetObjectError::NoSuchKey(_) => AppError::NonExistentCrateVersion {
                        crate_name: crate_name.to_string(),
                        version,
                    },
                    _ => anyhow::anyhow!("unexpected error in getting crate from S3").into(),
                };

                Err(mapped_err)
            }
            Ok(output) => output
                .body
                .collect()
                .await
                .map_err(|_| anyhow!("failed to collect bytes").into())
                .map(|data| data.into_bytes().to_vec()),
        }
    }
}
