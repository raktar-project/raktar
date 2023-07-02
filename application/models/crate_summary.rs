use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CrateSummary {
    pub name: String,
    #[serde(with = "serde_dynamo::number_set")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<u32>,
    pub max_version: Version,
    pub description: String,
}
