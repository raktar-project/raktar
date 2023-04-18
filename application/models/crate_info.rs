use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CrateInfo {
    #[serde(with = "serde_dynamo::number_set")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<u32>,
}
