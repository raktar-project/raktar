use crate::error::AppResult;
use crate::models::token::TokenItem;

#[async_trait::async_trait]
pub trait TokenRepository {
    async fn store_auth_token(
        &self,
        token: &[u8],
        name: String,
        user_id: u32,
    ) -> AppResult<TokenItem>;
    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()>;
    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<TokenItem>>;
    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<TokenItem>>;
}
