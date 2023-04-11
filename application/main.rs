mod api;
mod metadata;

use std::io::{Cursor, Read};

use crate::api::index::{
    get_info_for_long_name_crate, get_info_for_short_name_crate, get_info_for_three_letter_crate,
};
use aws_sdk_dynamodb::Client;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use byteorder::{LittleEndian, ReadBytesExt};
use lambda_web::run_hyper_on_lambda;
use serde::Serialize;
use serde_dynamo::to_item;
use tracing::{error, info};

use crate::metadata::Metadata;

#[derive(Serialize)]
struct PublishWarning {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

#[derive(Serialize)]
struct PublishResponse {
    warnings: Vec<PublishWarning>,
}

#[derive(Serialize)]
struct Config {
    dl: String,
    api: String,
}

async fn get_config_json() -> (StatusCode, Json<Config>) {
    let response = Config {
        dl: "https://23g9zd8v1b.execute-api.eu-west-1.amazonaws.com/api/v1/crates".to_string(),
        api: "https://23g9zd8v1b.execute-api.eu-west-1.amazonaws.com".to_string(),
    };

    (StatusCode::OK, Json(response))
}

async fn publish_crate(State(db_client): State<Client>, body: Bytes) -> Json<PublishResponse> {
    let mut cursor = Cursor::new(body);
    let metadata_length = cursor.read_u32::<LittleEndian>().unwrap();
    let mut metadata_bytes = vec![0u8; metadata_length as usize];
    cursor.read_exact(&mut metadata_bytes).unwrap();
    let metadata = serde_json::from_slice::<Metadata>(&metadata_bytes).unwrap();

    info!("metadata: {}", serde_json::to_string(&metadata).unwrap());
    let pk = aws_sdk_dynamodb::types::AttributeValue::S(metadata.name.clone());
    let sk = aws_sdk_dynamodb::types::AttributeValue::S(metadata.vers.to_string());
    let item = to_item(metadata).unwrap();
    match db_client
        .put_item()
        .table_name(get_table_name())
        .set_item(Some(item))
        .item("pk", pk)
        .item("sk", sk)
        .send()
        .await
    {
        Ok(_) => info!("successfully stored"),
        Err(err) => error!("{:?}", err),
    }

    let response = PublishResponse { warnings: vec![] };
    Json(response)
}

fn get_table_name() -> String {
    std::env::var("TABLE_NAME").unwrap()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();

    let aws_config = aws_config::from_env().load().await;
    let db_client = Client::new(&aws_config);

    let app = Router::new()
        .route("/config.json", get(get_config_json))
        .route("/api/v1/crates/new", put(publish_crate))
        .route("/1/:crate_name", get(get_info_for_short_name_crate))
        .route("/2/:crate_name", get(get_info_for_short_name_crate))
        .route(
            "/3/:first_letter/:crate_name",
            get(get_info_for_three_letter_crate),
        )
        .route(
            "/:first_two/:second_two/:crate_name",
            get(get_info_for_long_name_crate),
        )
        .with_state(db_client);

    run_app(app).await
}

#[cfg(feature = "local")]
async fn run_app(app: Router) {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3025));
    info!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("service to start successfully");
}

#[cfg(not(feature = "local"))]
async fn run_app(app: Router) {
    run_hyper_on_lambda(app)
        .await
        .expect("app to run on Lambda successfully")
}
