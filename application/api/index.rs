use axum::extract::Path;
use semver::Version;

use crate::models::index::PackageInfo;

pub async fn get_info_for_short_name_crate(Path(crate_name): Path<String>) -> String {
    assert_eq!(1, crate_name.len());

    get_crate_info(&crate_name).await
}

pub async fn get_info_for_three_letter_crate(
    Path((first_letter, crate_name)): Path<(String, String)>,
) -> String {
    assert_eq!(Some(first_letter.as_ref()), crate_name.get(0..1));

    get_crate_info(&crate_name).await
}

pub async fn get_info_for_long_name_crate(
    Path((first_two, second_two, crate_name)): Path<(String, String, String)>,
) -> String {
    assert_eq!(Some(first_two.as_ref()), crate_name.get(0..2));
    assert_eq!(Some(second_two.as_ref()), crate_name.get(2..4));

    get_crate_info(&crate_name).await
}

async fn get_crate_info(crate_name: &str) -> String {
    let info = PackageInfo {
        name: crate_name.to_string(),
        vers: Version {
            major: 0,
            minor: 1,
            patch: 0,
            pre: Default::default(),
            build: Default::default(),
        },
        deps: vec![],
        cksum: "".to_string(),
        features: Default::default(),
        yanked: false,
        links: None,
    };

    serde_json::to_string(&info).unwrap()
}
