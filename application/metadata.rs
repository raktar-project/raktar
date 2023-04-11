use std::collections::HashMap;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetadataDependency {
    pub name: String,
    pub version_req: VersionReq,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Option<String>,
    pub registry: Option<Url>,
    pub explicit_name_in_toml: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<MetadataDependency>,
    pub features: HashMap<String, Vec<String>>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<Url>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<Url>,
    pub badges: HashMap<String, HashMap<String, String>>,
    pub links: Option<String>,
    #[serde(default)]
    pub yanked: bool,
}
