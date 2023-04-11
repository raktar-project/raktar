mod metadata;

use aws_sdk_dynamodb::Client;
use byteorder::{LittleEndian, ReadBytesExt};
use poem::web::Data;
use poem::{Body, Endpoint, EndpointExt, Route};
use poem_lambda::Error;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi, OpenApiService};
use std::io::Read;
use tracing::info;

use crate::metadata::Metadata;

#[derive(Object)]
struct PublishWarning {
    invalid_categories: Vec<String>,
    invalid_badges: Vec<String>,
    other: Vec<String>,
}

#[derive(Object)]
struct PublishResponse {
    warnings: Vec<PublishWarning>,
}

#[derive(Object)]
struct ConfigResponse {
    dl: String,
    api: String,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/config.json", method = "get")]
    async fn config_json(&self) -> Json<ConfigResponse> {
        let response = ConfigResponse {
            dl: "https://23g9zd8v1b.execute-api.eu-west-1.amazonaws.com/api/v1/crates".to_string(),
            api: "https://23g9zd8v1b.execute-api.eu-west-1.amazonaws.com".to_string(),
        };

        Json(response)
    }

    #[oai(path = "/api/v1/crates/new", method = "put")]
    async fn publish_crate(&self, _db_client: Data<&Client>, body: Body) -> Json<PublishResponse> {
        let bytes = body.into_bytes().await.unwrap();
        let mut cursor = std::io::Cursor::new(bytes);
        let metadata_length = cursor.read_u32::<LittleEndian>().unwrap();
        let mut metadata_bytes = vec![0u8; metadata_length as usize];
        cursor.read_exact(&mut metadata_bytes).unwrap();
        let metadata = serde_json::from_slice::<Metadata>(&metadata_bytes).unwrap();

        info!("metadata: {}", serde_json::to_string(&metadata).unwrap());

        let response = PublishResponse { warnings: vec![] };
        Json(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().json().init();

    let aws_config = aws_config::from_env().load().await;
    let db_client = Client::new(&aws_config);

    let api_service = OpenApiService::new(Api, "Raktar", "1.0").server("");
    let ui = api_service.swagger_ui();
    let app = Route::new()
        .nest("/", api_service)
        .nest("/doc", ui)
        .data(db_client);

    run_app(app).await
}

#[cfg(feature = "local")]
async fn run_app(app: impl Endpoint + 'static) -> Result<(), Error> {
    let res = poem::Server::new(poem::listener::TcpListener::bind("127.0.0.1:3001"))
        .name("raktar-local")
        .run(app)
        .await;

    res.map_err(Error::from)
}

#[cfg(not(feature = "local"))]
async fn run_app(app: impl Endpoint + 'static) -> Result<(), Error> {
    poem_lambda::run(app).await
}
