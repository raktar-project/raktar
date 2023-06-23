use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use serde_json::{json, Value};
use tracing::error;

#[derive(Serialize)]
pub struct Config {
    dl: String,
    api: String,
    #[serde(rename = "auth-required")]
    auth_required: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Response {
    Config(Config),
    Error(Value),
}

pub async fn get_config_json() -> (StatusCode, Json<Response>) {
    if let Ok(domain_name) = std::env::var("DOMAIN_NAME") {
        let dl = format!("https://{}/api/v1/crates", domain_name);
        let api = format!("https://{}", domain_name);
        let config = Config {
            dl,
            api,
            auth_required: true,
        };

        (StatusCode::OK, Json(Response::Config(config)))
    } else {
        error!("DOMAIN_NAME is not set in environment");
        let error_response = json!({
            "reason": "misconfigured application"
        });
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response::Error(error_response)),
        )
    }
}
