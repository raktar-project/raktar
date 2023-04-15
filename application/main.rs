mod api;
pub mod app_state;
pub mod error;
pub mod models;
pub mod repository;
pub mod storage;

use aws_sdk_dynamodb::Client;
use axum::Router;
use lambda_web::run_hyper_on_lambda;

use crate::api::build_router;
use crate::app_state::AppState;
use crate::repository::DynamoDBRepository;
use crate::storage::S3Storage;

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
