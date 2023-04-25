mod config;
mod download;
mod index;
mod owners;
mod publish;
mod unyank;
mod yank;

use axum::routing::{delete, get, put, Router};

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
use crate::AppState;

pub fn build_router() -> Router<AppState> {
    Router::new()
        .route("/gql", get(graphiql).post(graphql_handler))
        .route("/config.json", get(get_config_json))
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
}
