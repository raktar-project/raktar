use crate::auth::AuthenticatedUser;
use anyhow::{anyhow, bail, Result};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;
use axum::response;
use axum::response::IntoResponse;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use http::HeaderMap;
use serde::Deserialize;
use std::str::FromStr;

use crate::graphql::schema::RaktarSchema;

pub async fn graphql_handler(
    schema: Extension<RaktarSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    match extract_user_id(&headers) {
        Ok(authenticated_user) => {
            let request = req.into_inner().data(authenticated_user);
            schema.execute(request).await.into()
        }
        Err(_) => {
            // In local, we allow requests with no authenticated user,
            // mostly to let the frontend to pull the schema.
            // TODO: revise this, we can probably do something smarter
            #[cfg(feature = "local")]
            return schema.execute(req.into_inner()).await.into();

            #[cfg(not(feature = "local"))]
            {
                let err = async_graphql::Error::new("failed to get claims from token")
                    .into_server_error(async_graphql::Pos { line: 0, column: 0 });
                let response = async_graphql::Response::from_errors(err.into());
                response.into()
            }
        }
    }
}

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/gql").finish())
}

#[derive(Debug, Deserialize)]
struct Claims {
    autogen_id: String,
}

fn extract_user_id(headers: &HeaderMap) -> Result<AuthenticatedUser> {
    headers
        .get("Authorization")
        .and_then(|header| {
            let token = header.to_str().ok()?;
            let claims = parse_token(token).ok()?;
            Some(AuthenticatedUser {
                id: u32::from_str(&claims.autogen_id).ok()?,
            })
        })
        .ok_or(anyhow!("failed to get authenticated user details"))
}

fn parse_token(token: &str) -> Result<Claims> {
    let mut parts = token.rsplitn(3, '.');
    if let (Some(_), Some(payload), Some(_), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    {
        let decoded = URL_SAFE_NO_PAD.decode(payload)?;
        let claims = serde_json::from_slice::<Claims>(&decoded)?;
        Ok(claims)
    } else {
        bail!("failed to parse token");
    }
}
