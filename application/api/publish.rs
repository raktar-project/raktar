use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use serde_dynamo::to_item;
use std::io::{Cursor, Read};
use tracing::{error, info};

use crate::app_state::AppState;
use crate::db::get_table_name;
use crate::metadata::Metadata;
use crate::storage::CrateStorage;

#[derive(Serialize)]
pub struct PublishWarning {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum PublishResponse {
    PublishSuccess { warnings: Vec<PublishWarning> },
    PublishFailure { detail: String },
}

pub async fn publish_crate<S: CrateStorage>(
    State(app_state): State<AppState<S>>,
    body: Bytes,
) -> (StatusCode, Json<PublishResponse>) {
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

    let pk = aws_sdk_dynamodb::types::AttributeValue::S(metadata.name.clone());
    let sk = aws_sdk_dynamodb::types::AttributeValue::S(vers.to_string());
    let item = to_item(metadata).unwrap();
    let (status_code, response) = match &app_state
        .db_client
        .put_item()
        .table_name(get_table_name())
        .set_item(Some(item))
        .item("pk", pk)
        .item("sk", sk)
        .condition_expression("attribute_not_exists(sk)")
        .send()
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            PublishResponse::PublishSuccess { warnings: vec![] },
        ),
        Err(err) => {
            error!("{:?}", err);
            let response = PublishResponse::PublishFailure {
                detail: format!("{}", err),
            };
            (StatusCode::BAD_REQUEST, response)
        }
    };
    app_state
        .storage
        .store_crate(&crate_name, vers, crate_bytes)
        .await
        .expect("to be able to store crate");

    (status_code, Json(response))
}
