use axum::extract::{Path, State};

use crate::app_state::AppState;
use crate::error::AppResult;
use crate::repository::Repository;
use crate::storage::CrateStorage;

pub async fn get_info_for_short_name_crate<R: Repository, S: CrateStorage>(
    Path(crate_name): Path<String>,
    State(app_state): State<AppState<R, S>>,
) -> AppResult<String> {
    assert_eq!(1, crate_name.len());

    app_state.repository.get_package_info(&crate_name).await
}

pub async fn get_info_for_three_letter_crate<R: Repository, S: CrateStorage>(
    Path((first_letter, crate_name)): Path<(String, String)>,
    State(app_state): State<AppState<R, S>>,
) -> AppResult<String> {
    assert_eq!(Some(first_letter.as_ref()), crate_name.get(0..1));

    app_state.repository.get_package_info(&crate_name).await
}

pub async fn get_info_for_long_name_crate<R: Repository, S: CrateStorage>(
    State(app_state): State<AppState<R, S>>,
    Path((first_two, second_two, crate_name)): Path<(String, String, String)>,
) -> AppResult<String> {
    assert_eq!(Some(first_two.as_ref()), crate_name.get(0..2));
    assert_eq!(Some(second_two.as_ref()), crate_name.get(2..4));

    app_state.repository.get_package_info(&crate_name).await
}
