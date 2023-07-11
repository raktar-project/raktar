//! The /me API that Cargo directs you to when you do `cargo login`.
//!
//! Because we don't actually generate the token directly here,
//! we just redirect to the frontend applications tokens page.
use anyhow::{anyhow, bail};
use axum::response::{IntoResponse, Redirect, Response};
use http::{HeaderMap, StatusCode};
use tracing::error;

pub async fn redirect_for_token(headers: HeaderMap) -> Response {
    match get_host_from_headers(&headers) {
        Ok(host) => match get_tokens_url_from_host(host) {
            Ok(url) => return Redirect::to(&url).into_response(),
            Err(err) => {
                let error_message = err.to_string();
                error!(error_message, host, "failed to get URL from host");
            }
        },
        Err(err) => {
            let error_message = err.to_string();
            error!(error_message, "failed to get host from headers");
        }
    }

    (StatusCode::NOT_FOUND, "not found").into_response()
}

fn get_host_from_headers(headers: &HeaderMap) -> anyhow::Result<&str> {
    if let Some(header) = headers.get("host") {
        let host = header.to_str()?;
        Ok(host)
    } else {
        bail!("headers did not contain host")
    }
}

fn get_tokens_url_from_host(host: &str) -> anyhow::Result<String> {
    let app_host = host
        .strip_prefix("api.")
        .ok_or(anyhow!("host does not start with api"))?;

    Ok(format!("https://{app_host}/tokens"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn test_get_tokens_url_from_host() {
        let tokens_url = get_tokens_url_from_host("api.raktar.io").unwrap();
        assert_eq!(tokens_url, "https://raktar.io/tokens");
    }

    #[test]
    fn test_get_host_from_headers() {
        let mut headers = HeaderMap::new();
        let host_header = HeaderValue::from_str("api.raktar.io").unwrap();
        headers.insert("host", host_header);

        let host = get_host_from_headers(&headers).unwrap();
        assert_eq!(host, "api.raktar.io");
    }
}
