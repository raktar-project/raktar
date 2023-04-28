use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;
use axum::response;
use axum::response::IntoResponse;

use crate::graphql::schema::RaktarSchema;

pub async fn graphql_handler(
    schema: Extension<RaktarSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/gql").finish())
}
