use aws_sdk_dynamodb::Client;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;
use serde_dynamo::to_item;
use std::io::{Cursor, Read};
use tracing::{error, info};

use crate::db::get_table_name;
use crate::metadata::Metadata;

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

pub async fn publish_crate(
    State(db_client): State<Client>,
    body: Bytes,
) -> (StatusCode, Json<PublishResponse>) {
    let mut cursor = Cursor::new(body);
    let metadata_length = cursor.read_u32::<LittleEndian>().unwrap();
    let mut metadata_bytes = vec![0u8; metadata_length as usize];
    cursor.read_exact(&mut metadata_bytes).unwrap();
    let metadata = serde_json::from_slice::<Metadata>(&metadata_bytes).unwrap();

    info!("metadata: {}", serde_json::to_string(&metadata).unwrap());
    let pk = aws_sdk_dynamodb::types::AttributeValue::S(metadata.name.clone());
    let sk = aws_sdk_dynamodb::types::AttributeValue::S(metadata.vers.to_string());
    let item = to_item(metadata).unwrap();
    let (status_code, response) = match db_client
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

    (status_code, Json(response))
}
