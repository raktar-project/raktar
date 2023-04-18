use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: u32,
    pub login: String,
    pub name: Option<String>,
}
