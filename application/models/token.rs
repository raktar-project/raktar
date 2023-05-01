use crate::auth::hash;
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenItem {
    pub pk: String,
    pub sk: String,
    pub name: String,
    pub user_id: u32,
    pub token_id: String,
}

impl TokenItem {
    pub fn new(token: &[u8], name: String, user_id: u32) -> Self {
        Self {
            pk: Self::get_pk(token),
            sk: Self::get_sk(),
            name,
            user_id,
            token_id: Uuid::new_v4().hyphenated().to_string(),
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
