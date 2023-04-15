mod config;
mod download;
mod index;
mod publish;
mod unyank;
mod yank;

use axum::routing::{delete, get, put, Router};

use crate::api::config::get_config_json;
use crate::api::download::download_crate;
use crate::api::index::{
    get_info_for_long_name_crate, get_info_for_short_name_crate, get_info_for_three_letter_crate,
};
use crate::api::publish::publish_crate;
use crate::api::unyank::unyank;
use crate::api::yank::yank;
use crate::app_state::AppState;
use crate::repository::Repository;
use crate::storage::CrateStorage;

pub fn build_router<R: Repository, S: CrateStorage>() -> Router<AppState<R, S>> {
    Router::new()
        .route("/config.json", get(get_config_json))
        .route("/api/v1/crates/new", put(publish_crate))
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
