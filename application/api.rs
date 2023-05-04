mod config;
mod download;
mod index;
mod owners;
mod publish;
mod unyank;
mod yank;

use axum::routing::{delete, get, put, Router};
use axum::Extension;
use raktar::auth::token_authenticator;
use raktar::repository::DynRepository;

use crate::api::config::get_config_json;
use crate::api::download::download_crate;
use crate::api::index::{
    get_info_for_long_name_crate, get_info_for_short_name_crate, get_info_for_three_letter_crate,
};
use crate::api::owners::{add_owners, list_owners};
use crate::api::publish::publish_crate;
use crate::api::unyank::unyank;
use crate::api::yank::yank;
use crate::graphql::handler::{graphiql, graphql_handler};
use crate::graphql::schema::build_schema;
use crate::storage::DynCrateStorage;
use crate::AppState;

pub fn build_router(repository: DynRepository, storage: DynCrateStorage) -> Router {
    let core_router = build_core_router(repository.clone());
    let graphql_router = build_graphql_router(repository.clone());
    let state = (repository, storage);

    Router::new()
        .route("/config.json", get(get_config_json))
        .nest("/", core_router)
        .nest("/gql", graphql_router)
        .with_state(state)
}

fn build_core_router(repository: DynRepository) -> Router<AppState> {
    Router::new()
        .route("/api/v1/crates/new", put(publish_crate))
        .route(
            "/api/v1/crates/:crate_name/owners",
            get(list_owners).put(add_owners),
        )
        .route("/api/v1/crates/:crate_name/:version/yank", delete(yank))
        .route("/api/v1/crates/:crate_name/:version/unyank", put(unyank))
        .route(
            "/api/v1/crates/:crate_name/:version/download",
            get(download_crate),
        )
        .route("/1/:crate_name", get(get_info_for_short_name_crate))
        .route("/2/:crate_name", get(get_info_for_short_name_crate))
        .route(
            "/3/:first_letter/:crate_name",
            get(get_info_for_three_letter_crate),
        )
        .route(
            "/:first_two/:second_two/:crate_name",
            get(get_info_for_long_name_crate),
        )
        .layer(axum::middleware::from_fn_with_state(
            repository,
            token_authenticator,
        ))
}

fn build_graphql_router(repository: DynRepository) -> Router<AppState> {
    let schema = build_schema(repository);
    Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
}
