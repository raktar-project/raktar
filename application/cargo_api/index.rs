use axum::extract::{Path, State};

use crate::error::AppResult;
use crate::router::AppState;

pub async fn get_info_for_short_name_crate(
    Path(crate_name): Path<String>,
    State((repository, _)): State<AppState>,
) -> AppResult<String> {
    assert_eq!(1, crate_name.len());

    repository.get_package_info(&crate_name).await
}

pub async fn get_info_for_three_letter_crate(
    Path((first_letter, crate_name)): Path<(String, String)>,
    State((repository, _)): State<AppState>,
) -> AppResult<String> {
    assert_eq!(Some(first_letter.as_ref()), crate_name.get(0..1));

    repository.get_package_info(&crate_name).await
}

pub async fn get_info_for_long_name_crate(
    Path((first_two, second_two, crate_name)): Path<(String, String, String)>,
    State((repository, _)): State<AppState>,
) -> AppResult<String> {
    assert_eq!(Some(first_two.as_ref()), crate_name.get(0..2));
    assert_eq!(Some(second_two.as_ref()), crate_name.get(2..4));

    repository.get_package_info(&crate_name).await
}
