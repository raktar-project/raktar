use crate::error::AppResult;
use crate::models::token::Token;

#[async_trait::async_trait]
pub trait TokenRepository {
    async fn store_auth_token(&self, token: &[u8], name: String, user_id: u32) -> AppResult<Token>;
    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()>;
    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<Token>>;
    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<Token>>;
}
