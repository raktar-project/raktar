use crate::error::AppResult;
use crate::models::user::{CognitoUserData, User, UserId};

#[async_trait::async_trait]
pub trait UserRepository {
    /// Used by the pre-token Lambda to ensure the SSO user is up to date in the repository.
    ///
    /// This either creates a new user, or checks whether the existing user has the
    /// latest data (e.g. first name and last name being up to date) and bring the
    /// database in line if it's out of sync.
    async fn update_or_create_user(&self, user_data: CognitoUserData) -> AppResult<User>;
    async fn get_user_by_id(&self, user_id: UserId) -> AppResult<Option<User>>;
    async fn get_users(&self) -> AppResult<Vec<User>>;
}
