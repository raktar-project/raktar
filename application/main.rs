use poem::Route;
use poem_lambda::Error;
use poem_openapi::payload::PlainText;
use poem_openapi::{OpenApi, OpenApiService};

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/", method = "get")]
    async fn api_list(&self) -> PlainText<String> {
        PlainText("hello world".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().json().init();

    let api_service = OpenApiService::new(Api, "ApiView", "1.0").server("");
    let ui = api_service.swagger_ui();
    let app = Route::new().nest("/", api_service).nest("/doc", ui);

    run_app(app).await
}

#[cfg(feature = "local")]
async fn run_app(app: Route) -> Result<(), Error> {
    let res = poem::Server::new(poem::listener::TcpListener::bind("127.0.0.1:3001"))
        .name("apiview-local")
        .run(app)
        .await;

    res.map_err(Error::from)
}

#[cfg(not(feature = "local"))]
async fn run_app(app: Route) -> Result<(), Error> {
    poem_lambda::run(app).await
}
