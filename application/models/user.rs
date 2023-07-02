use serde::{Deserialize, Serialize};

pub type UserId = u32;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: UserId,
    pub login: String,
    #[serde(default)]
    pub name: Option<String>,
}
