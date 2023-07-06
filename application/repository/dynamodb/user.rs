use crate::error::AppResult;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use serde_dynamo::from_items;

use crate::models::user::{User, UserId};

pub async fn get_user_by_id(
    db_client: &Client,
    table_name: &str,
    user_id: UserId,
) -> AppResult<Option<User>> {
    let sk = format!("ID#{:06}", user_id);
    let output = db_client
        .get_item()
        .table_name(table_name)
        .key("pk", AttributeValue::S("USERS".to_string()))
        .key("sk", AttributeValue::S(sk))
        .send()
        .await?;

    let user = if let Some(item) = output.item().cloned() {
        Some(serde_dynamo::from_item(item)?)
    } else {
        None
    };

    Ok(user)
}
pub async fn get_users(db_client: &Client, table_name: &str) -> AppResult<Vec<User>> {
    let output = db_client
        .query()
        .table_name(table_name)
        .key_condition_expression("pk = :pk and begins_with(sk, :prefix)")
        .expression_attribute_values(":pk", AttributeValue::S("USERS".to_string()))
        .expression_attribute_values(":prefix", AttributeValue::S("ID#".to_string()))
        .send()
        .await?;

    match output.items() {
        None => Ok(vec![]),
        Some(items) => {
            let users = from_items(items.to_vec())?;
            Ok(users)
        }
    }
}
