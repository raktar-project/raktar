mod api;
pub mod app_state;
pub mod error;
pub mod models;
pub mod repository;
pub mod storage;

use crate::api::build_router;
use aws_sdk_dynamodb::Client;
use axum::http::StatusCode;
use axum::{Json, Router};
use lambda_web::run_hyper_on_lambda;
use serde::Serialize;

use crate::app_state::AppState;
use crate::repository::DynamoDBRepository;
use crate::storage::S3Storage;

#[derive(Serialize)]
struct Config {
    dl: String,
    api: String,
}

async fn get_config_json() -> (StatusCode, Json<Config>) {
    let domain_name = std::env::var("DOMAIN_NAME").unwrap();
    let dl = format!("https://{}/api/v1/crates", domain_name);
    let api = format!("https://{}", domain_name);
    let response = Config { dl, api };

    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();

    let aws_config = aws_config::from_env().load().await;
    let db_client = Client::new(&aws_config);
    let repository = DynamoDBRepository::new(db_client);
    let storage = S3Storage::new().await;
    let app_state = AppState {
        repository,
        storage,
    };

    let app = build_router().with_state(app_state);

    run_app(app).await
}

#[cfg(feature = "local")]
async fn run_app(app: Router) {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3025));
    tracing::info!("listening on http://{}", addr);
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
