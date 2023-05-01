use serde::{Deserialize, Serialize};

pub type UserId = u32;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    pub login: String,
    pub name: Option<String>,
}
