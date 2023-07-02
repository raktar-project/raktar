use std::sync::Arc;

use aws_sdk_dynamodb::Client;
use axum::Router;
use raktar::api::build_router;
use raktar::repository::{DynRepository, DynamoDBRepository};
use raktar::storage::{DynCrateStorage, S3Storage};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();

    let aws_config = aws_config::from_env().load().await;
    let db_client = Client::new(&aws_config);
    let repository = Arc::new(DynamoDBRepository::new_from_env(db_client)) as DynRepository;
    let storage = Arc::new(S3Storage::new().await) as DynCrateStorage;

    let app = build_router(repository, storage);

    run_app(app).await
}

#[cfg(feature = "local")]
async fn run_app(app: Router) {
    let cors_layer = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);
    let app_with_cors = app.layer(cors_layer);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3026));
    tracing::info!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app_with_cors.into_make_service())
        .await
        .expect("service to start successfully");
}

#[cfg(not(feature = "local"))]
async fn run_app(app: Router) {
    lambda_web::run_hyper_on_lambda(app)
        .await
        .expect("app to run on Lambda successfully")
}
