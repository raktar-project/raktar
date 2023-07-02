use crate::error::AppResult;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;

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
