use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenItem {
    #[serde(rename = "pk")]
    id: String,
    #[serde(rename = "sk")]
    name: String,
    user: u32,
}

impl TokenItem {
    pub(crate) fn new(token: Vec<u8>, name: String) -> Self {
        let encoded = base64::engine::general_purpose::STANDARD.encode(token);
        Self {
            id: format!("TOK#{}", encoded),
            user: 0,
            name,
        }
    }
}
