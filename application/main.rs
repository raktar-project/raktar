mod api;
pub mod app_state;
pub mod error;
pub mod models;
pub mod repository;
pub mod storage;

use aws_sdk_dynamodb::Client;
use axum::http::StatusCode;
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use lambda_web::run_hyper_on_lambda;
use serde::Serialize;

use crate::api::download::download_crate;
use crate::api::index::{
    get_info_for_long_name_crate, get_info_for_short_name_crate, get_info_for_three_letter_crate,
};
use crate::api::publish::publish_crate;
use crate::api::unyank::unyank;
use crate::api::yank::yank;
use crate::app_state::AppState;
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
    let storage = S3Storage::new().await;
    let app_state = AppState { db_client, storage };

    let app = Router::new()
        .route("/config.json", get(get_config_json))
        .route("/api/v1/crates/new", put(publish_crate))
        .route("/api/v1/crates/:crate_name/:version/yank", delete(yank))
        .route("/api/v1/crates/:crate_name/:version/unyank", put(unyank))
        .route(
            "/api/v1/crates/:crate_name/:version/download",
            get(download_crate),
        )
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
        .with_state(app_state);

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
