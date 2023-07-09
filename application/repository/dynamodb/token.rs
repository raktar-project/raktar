use aws_sdk_dynamodb::types::AttributeValue;
use serde_dynamo::{from_item, from_items, to_item};

use crate::error::AppResult;
use crate::models::token::TokenItem;
use crate::repository::base::TokenRepository;
use crate::repository::DynamoDBRepository;

#[async_trait::async_trait]
impl TokenRepository for DynamoDBRepository {
    async fn store_auth_token(
        &self,
        token: &[u8],
        name: String,
        user_id: u32,
    ) -> AppResult<TokenItem> {
        let token_item = TokenItem::new(token, name, user_id);
        let item = to_item(token_item.clone())?;
        self.db_client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;

        Ok(token_item)
    }

    async fn delete_auth_token(&self, user_id: u32, token_id: String) -> AppResult<()> {
        let tokens = self.list_auth_tokens(user_id).await?;
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

    async fn list_auth_tokens(&self, user_id: u32) -> AppResult<Vec<TokenItem>> {
        // TODO: this shouldn't return TokenItem
        // in fact, we shouldn't leak TokenItem at all, as it's a DynamoDB model
        let output = self
            .db_client
            .query()
            .table_name(&self.table_name)
            .index_name("user_tokens")
            .key_condition_expression("user_id = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::N(user_id.to_string()))
            .send()
            .await?;

        let items = output.items().map(|items| items.to_vec()).unwrap_or(vec![]);
        let tokens = from_items(items)?;

        Ok(tokens)
    }

    async fn get_auth_token(&self, token: &[u8]) -> AppResult<Option<TokenItem>> {
        let output = self
            .db_client
            .get_item()
            .table_name(&self.table_name)
            .key("pk", AttributeValue::S(TokenItem::get_pk(token)))
            .key("sk", AttributeValue::S(TokenItem::get_sk()))
            .send()
            .await?;

        let token_item = if let Some(item) = output.item().cloned() {
            Some(from_item(item)?)
        } else {
            None
        };

        Ok(token_item)
    }
}
