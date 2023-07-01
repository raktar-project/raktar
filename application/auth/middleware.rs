use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use tracing::{error, warn};

use crate::auth::AuthenticatedUser;
use crate::repository::DynRepository;

pub async fn token_authenticator<B>(
    State(repository): State<DynRepository>,
    mut request: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    if let Some(auth_header) = request.headers().get("Authorization") {
        let token = auth_header.as_bytes();
        match repository.get_auth_token(token).await {
            Ok(Some(t)) => {
                let user = AuthenticatedUser { id: t.user_id };
                request.extensions_mut().insert(user);
                return next.run(request).await;
            }
            Err(err) => {
                error!(
                    err = err.to_string(),
                    "error in trying to get token for user"
                );
            }
            _ => {}
        }
    }

    warn!("unauthorized attempt to access registry");
    (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response()
}
