use serde::{Deserialize, Serialize};

pub type UserId = u32;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: UserId,
    pub login: String,
    pub given_name: String,
    pub family_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CognitoUserData {
    pub login: String,
    pub given_name: String,
    pub family_name: String,
}

impl CognitoUserData {
    pub fn into_user(self, id: UserId) -> User {
        User {
            id,
            login: self.login,
            given_name: self.given_name,
            family_name: self.family_name,
        }
    }
}
