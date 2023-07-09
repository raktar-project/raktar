use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use serde_dynamo::{from_item, from_items, to_item};
use std::str::FromStr;
use tracing::info;

use crate::error::{internal_error, AppResult};
use crate::models::user::{CognitoUserData, User, UserId};

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

pub async fn put_user(
    db_client: &Client,
    table_name: &str,
    user: User,
    is_new: bool,
) -> AppResult<User> {
    let condition = if is_new {
        Some("attribute_not_exists(pk) AND attribute_not_exists(sk)".to_string())
    } else {
        None
    };
    let put = Put::builder()
        .table_name(table_name)
        .set_item(Some(to_item(user.clone())?))
        .item("pk", AttributeValue::S("USERS".to_string()))
        .item("sk", AttributeValue::S(format!("LOGIN#{}", user.login)))
        .set_condition_expression(condition)
        .build();
    let put_login_mapping_item = TransactWriteItem::builder().put(put).build();

    let user_id_sk = AttributeValue::S(format!("ID#{:06}", user.id.clone()));
    let put = Put::builder()
        .table_name(table_name)
        .set_item(Some(to_item(user.clone())?))
        .item("pk", AttributeValue::S("USERS".to_string()))
        .item("sk", user_id_sk)
        .build();
    let put_user_item = TransactWriteItem::builder().put(put).build();

    db_client
        .transact_write_items()
        .transact_items(put_login_mapping_item)
        .transact_items(put_user_item)
        .send()
        .await?;

    Ok(user)
}

pub async fn find_next_user_id(db_client: &Client, table_name: &str) -> AppResult<u32> {
    let output = db_client
        .query()
        .table_name(table_name)
        .key_condition_expression("pk = :pk AND begins_with(sk, :prefix)")
        .expression_attribute_values(":pk", AttributeValue::S("USERS".to_string()))
        .expression_attribute_values(":prefix", AttributeValue::S("ID#".to_string()))
        .scan_index_forward(false)
        .send()
        .await?;

    let current_id = if let Some(item) = output.items().and_then(|items| items.iter().next()) {
        let attr = item.get("id").ok_or(internal_error())?;
        let id_string = attr.as_n().map_err(|_| internal_error())?;
        u32::from_str(id_string)?
    } else {
        0
    };

    Ok(current_id + 1)
}

pub async fn create_next_user(
    db_client: &Client,
    table_name: &str,
    user_data: CognitoUserData,
) -> AppResult<User> {
    let next_id = find_next_user_id(db_client, table_name).await?;
    info!("next available ID is {}", next_id);

    let user = user_data.into_user(next_id);

    put_user(db_client, table_name, user, true).await
}

pub async fn update_or_create_user(
    db_client: &Client,
    table_name: &str,
    user_data: CognitoUserData,
) -> AppResult<User> {
    let output = db_client
        .get_item()
        .table_name(table_name)
        .key("pk", AttributeValue::S("USERS".to_string()))
        .key(
            "sk",
            AttributeValue::S(format!("LOGIN#{}", user_data.login)),
        )
        .send()
        .await?;

    match output.item().cloned() {
        None => {
            info!("user not found, creating new user");
            create_next_user(db_client, table_name, user_data).await
        }
        Some(item) => {
            let user: User = from_item(item)?;

            // if the existing user data is out of sync, update it
            let existing_user_data: CognitoUserData = user.clone().into();
            if existing_user_data != user_data {
                let new_user = user_data.into_user(user.id);
                put_user(db_client, table_name, new_user, false).await?;
            }

            Ok(user)
        }
    }
}
