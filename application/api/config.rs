use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Config {
    dl: String,
    api: String,
    #[serde(rename = "auth-required")]
    auth_required: bool,
}

pub async fn get_config_json() -> (StatusCode, Json<Config>) {
    let domain_name = std::env::var("DOMAIN_NAME").unwrap();
    let dl = format!("https://{}/api/v1/crates", domain_name);
    let api = format!("https://{}", domain_name);
    let response = Config {
        dl,
        api,
        auth_required: true,
    };

    (StatusCode::OK, Json(response))
}
