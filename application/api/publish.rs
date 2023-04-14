use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::Client;
use axum::body::Bytes;
use axum::extract::State;
use axum::Json;
use byteorder::{LittleEndian, ReadBytesExt};
use hex::ToHex;
use semver::Version;
use serde::Serialize;
use serde_dynamo::to_item;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use tracing::{error, info};

use crate::app_state::AppState;
use crate::db::get_table_name;
use crate::error::{AppError, AppResult};
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::storage::CrateStorage;

#[derive(Serialize)]
pub struct PublishResponse {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

pub async fn publish_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    body: Bytes,
) -> AppResult<Json<PublishResponse>> {
    let mut cursor = Cursor::new(body);

    // read metadata bytes
    let metadata_length = cursor.read_u32::<LittleEndian>().unwrap();
    let mut metadata_bytes = vec![0u8; metadata_length as usize];
    cursor.read_exact(&mut metadata_bytes).unwrap();
    let metadata = serde_json::from_slice::<Metadata>(&metadata_bytes).unwrap();

    // read crate bytes
    let crate_length = cursor.read_u32::<LittleEndian>().unwrap();
    let mut crate_bytes = vec![0u8; crate_length as usize];
    cursor.read_exact(&mut crate_bytes).unwrap();

    info!("metadata: {}", serde_json::to_string(&metadata).unwrap());
    let vers = metadata.vers.clone();
    let crate_name = metadata.name.clone();
    let checksum: String = Sha256::digest(&crate_bytes).encode_hex();
    let package_info = PackageInfo::from_metadata(metadata, &checksum);

    store_package_info(&app_state.db_client, &crate_name, &vers, package_info).await?;
    app_state
        .storage
        .store_crate(&crate_name, vers, crate_bytes)
        .await
        .map(|_| {
            Json(PublishResponse {
                invalid_categories: vec![],
                invalid_badges: vec![],
                other: vec![],
            })
        })
        .map_err(Into::into)
}

async fn store_package_info(
    db_client: &Client,
    crate_name: &str,
    version: &Version,
    package_info: PackageInfo,
) -> AppResult<()> {
    let pk = aws_sdk_dynamodb::types::AttributeValue::S(package_info.name.clone());
    let sk = aws_sdk_dynamodb::types::AttributeValue::S(package_info.vers.to_string());

    let item = to_item(package_info).unwrap();
    match db_client
        .put_item()
        .table_name(get_table_name())
        .set_item(Some(item))
        .item("pk", pk)
        .item("sk", sk)
        .condition_expression("attribute_not_exists(sk)")
        .send()
        .await
    {
        Ok(_) => {
            info!(
                crate_name = crate_name,
                version = version.to_string(),
                "persisted package info"
            );
            Ok(())
        }
        Err(err) => {
            let err = match err.into_service_error() {
                PutItemError::ConditionalCheckFailedException(_) => {
                    AppError::DuplicateCrateVersion {
                        crate_name: crate_name.to_string(),
                        version: version.clone(),
                    }
                }
                _ => {
                    error!("failed to store package info");
                    anyhow::anyhow!("unexpected error in persisting crate").into()
                }
            };

            Err(err)
        }
    }
}
