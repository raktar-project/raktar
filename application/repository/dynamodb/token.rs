use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_item, from_items, to_item};
use uuid::Uuid;

use crate::auth::hash;
use crate::error::AppResult;
use crate::models::token::Token;
use crate::models::user::UserId;
use crate::repository::base::TokenRepository;
use crate::repository::DynamoDBRepository;

#[async_trait::async_trait]
impl TokenRepository for DynamoDBRepository {
    async fn store_auth_token(&self, token: &[u8], name: String, user_id: u32) -> AppResult<Token> {
        let token_item = TokenItem::new(token, name, user_id);
        let item = to_item(token_item.clone())?;
        self.db_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;

        Ok(token_item.into())
    }

    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()> {
        let tokens =
            TokenItem::get_tokens_for_user(&self.db_client, &self.table_name, user_id).await?;
        if let Some(token_to_delete) = tokens.into_iter().find(|item| item.token_id == token_id) {
            self.db_client
                .delete_item()
                .table_name(&self.table_name)
                .key("pk", AttributeValue::S(token_to_delete.pk))
                .key("sk", AttributeValue::S(token_to_delete.sk))
                .send()
                .await?;
        }

        Ok(())
    }

    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<Token>> {
        let token_items =
            TokenItem::get_tokens_for_user(&self.db_client, &self.table_name, user_id).await?;
        let tokens = token_items.into_iter().map(|i| i.into()).collect();

        Ok(tokens)
    }

    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<Token>> {
        let output = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(TokenItem::get_pk(token)))
            .key("sk", AttributeValue::S(TokenItem::get_sk()))
            .send()
            .await?;

        let token = if let Some(item) = output.item().cloned() {
            let token_item: TokenItem = from_item(item)?;
            Some(token_item.into())
        } else {
            None
        };

        Ok(token)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TokenItem {
    pub pk: String,
    pub sk: String,
    pub name: String,
    pub user_id: u32,
    pub token_id: String,
}

impl TokenItem {
    fn new(token: &[u8], name: String, user_id: u32) -> Self {
        Self {
            pk: Self::get_pk(token),
            sk: Self::get_sk(),
            name,
            user_id,
            token_id: Uuid::new_v4().hyphenated().to_string(),
        }
    }

    async fn get_tokens_for_user(
        db_client: &Client,
        table_name: &str,
        user_id: UserId,
    ) -> AppResult<Vec<TokenItem>> {
        let output = db_client
            .query()
            .table_name(table_name)
            .index_name("user_tokens")
            .key_condition_expression("user_id = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::N(user_id.to_string()))
            .send()
            .await?;

        let items = output.items().map(|items| items.to_vec()).unwrap_or(vec![]);
        Ok(from_items(items)?)
    }

    fn get_pk(token: &[u8]) -> String {
        let encoded = base64::engine::general_purpose::STANDARD.encode(hash(token));
        format!("TOK#{}", encoded)
    }

    fn get_sk() -> String {
        "TOK".to_string()
    }
}

impl From<TokenItem> for Token {
    fn from(item: TokenItem) -> Self {
        Self {
            name: item.name,
            user_id: item.user_id,
            token_id: item.token_id,
        }
    }
}
