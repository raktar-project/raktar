use axum::body::Bytes;
use axum::extract::State;
use axum::{Extension, Json};
use byteorder::{LittleEndian, ReadBytesExt};
use hex::ToHex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use tracing::info;

use crate::auth::AuthenticatedUser;
use crate::error::AppResult;
use crate::models::index::PackageInfo;
use crate::models::metadata::Metadata;
use crate::repository::DynRepository;
use crate::router::AppState;
use crate::storage::DynCrateStorage;

#[derive(Serialize)]
pub struct PublishResponse {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

pub async fn publish_crate_handler(
    Extension(authenticated_user): Extension<AuthenticatedUser>,
    State((repository, storage)): State<AppState>,
    body: Bytes,
) -> AppResult<Json<PublishResponse>> {
    publish_crate(authenticated_user, storage, repository, body).await?;

    Ok(Json(PublishResponse {
        invalid_categories: vec![],
        invalid_badges: vec![],
        other: vec![],
    }))
}

pub async fn publish_crate(
    authenticated_user: AuthenticatedUser,
    storage: DynCrateStorage,
    repository: DynRepository,
    data: Bytes,
) -> AppResult<()> {
    let (metadata_bytes, crate_bytes) = read_body(data);
    let metadata = serde_json::from_slice::<Metadata>(&metadata_bytes).unwrap();

    info!("metadata: {}", serde_json::to_string(&metadata).unwrap());
    let vers = metadata.vers.clone();
    let crate_name = metadata.name.clone();
    let checksum: String = Sha256::digest(&crate_bytes).encode_hex();
    let package_info = PackageInfo::from_metadata(metadata.clone(), &checksum);

    info!(
        crate_name,
        vers = vers.to_string(),
        user_id = authenticated_user.id,
        "publishing new crate: {} {}",
        crate_name,
        vers
    );

    repository
        .store_package_info(
            &crate_name,
            &vers,
            package_info,
            metadata,
            &authenticated_user,
        )
        .await?;
    storage.store_crate(&crate_name, vers, crate_bytes).await?;

    Ok(())
}

fn read_body(body: Bytes) -> (Vec<u8>, Vec<u8>) {
    let mut cursor = Cursor::new(body);

    // read metadata bytes
    let metadata_length = cursor.read_u32::<LittleEndian>().unwrap();
    let mut metadata_bytes = vec![0u8; metadata_length as usize];
    cursor.read_exact(&mut metadata_bytes).unwrap();

    // read crate bytes
    let crate_length = cursor.read_u32::<LittleEndian>().unwrap();
    let mut crate_bytes = vec![0u8; crate_length as usize];
    cursor.read_exact(&mut crate_bytes).unwrap();

    (metadata_bytes, crate_bytes)
}
