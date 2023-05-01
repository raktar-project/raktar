use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use tracing::warn;

use crate::repository::DynRepository;

pub async fn token_authenticator<B>(
    State(repository): State<DynRepository>,
    request: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    if let Some(auth_header) = request.headers().get("Authorization") {
        let token = auth_header.as_bytes();
        if repository
            .get_auth_token(token)
            .await
            .map_or(false, |item| item.is_some())
        {
            return next.run(request).await;
        }
    }

    warn!("unauthorized attempt to access registry");
    (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response()
}
