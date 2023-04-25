mod api;
pub mod error;
mod graphql;
pub mod models;
pub mod repository;
pub mod storage;

use aws_sdk_dynamodb::Client;
use axum::{Extension, Router};
use std::sync::Arc;

use crate::api::build_router;
use crate::graphql::schema::build_schema;
use crate::repository::{DynRepository, DynamoDBRepository};
use crate::storage::{DynCrateStorage, S3Storage};

pub type AppState = (DynRepository, DynCrateStorage);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();

    let schema = build_schema();
    let aws_config = aws_config::from_env().load().await;
    let db_client = Client::new(&aws_config);
    let repository = Arc::new(DynamoDBRepository::new(db_client)) as DynRepository;
    let storage = Arc::new(S3Storage::new().await) as DynCrateStorage;

    let app = build_router()
        .layer(Extension(schema))
        .with_state((repository, storage));

    run_app(app).await
}

#[cfg(feature = "local")]
async fn run_app(app: Router) {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3026));
    tracing::info!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("service to start successfully");
}

#[cfg(not(feature = "local"))]
async fn run_app(app: Router) {
    lambda_web::run_hyper_on_lambda(app)
        .await
        .expect("app to run on Lambda successfully")
}
