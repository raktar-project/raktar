use crate::auth::hash;
use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenItem {
    pk: String,
    sk: String,
    name: String,
    user: u32,
}

impl TokenItem {
    pub fn new(token: &[u8], name: String) -> Self {
        Self {
            pk: Self::get_pk(token),
            sk: Self::get_sk(),
            name,
            user: 0,
        }
    }

    pub fn get_pk(token: &[u8]) -> String {
        let encoded = base64::engine::general_purpose::STANDARD.encode(hash(token));
        format!("TOK#{}", encoded)
    }

    pub fn get_sk() -> String {
        "TOK".to_string()
    }
}
