use axum::extract::Path;
use semver::Version;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct Dependency {}

/// The package information returned from the index as described in the Cargo reference:
/// https://doc.rust-lang.org/cargo/reference/registry-index.html
#[derive(Serialize)]
struct CrateVersionInfo {
    name: String,
    vers: Version,
    deps: Vec<Dependency>,
    cksum: String,
    features: HashMap<String, Vec<String>>,
    yanked: bool,
    links: Option<String>,
}

pub async fn get_info_for_short_name_crate(Path(crate_name): Path<String>) -> String {
    let info = CrateVersionInfo {
        name: crate_name,
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

pub async fn get_info_for_three_letter_crate(
    Path((first_letter, crate_name)): Path<(String, String)>,
) -> String {
    assert_eq!(Some(first_letter.as_ref()), crate_name.get(0..1));
    let info = CrateVersionInfo {
        name: crate_name,
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

pub async fn get_info_for_long_name_crate(
    Path((first_two, second_two, crate_name)): Path<(String, String, String)>,
) -> String {
    assert_eq!(Some(first_two.as_ref()), crate_name.get(0..2));
    assert_eq!(Some(second_two.as_ref()), crate_name.get(2..4));
    let info = CrateVersionInfo {
        name: crate_name,
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
