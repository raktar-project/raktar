mod krate;
mod token;
pub mod user;

use aws_sdk_dynamodb::Client;

use crate::repository::Repository;

#[derive(Clone)]
pub struct DynamoDBRepository {
    db_client: Client,
    table_name: String,
}

impl DynamoDBRepository {
    pub fn new(db_client: Client, table_name: String) -> Self {
        Self {
            db_client,
            table_name,
        }
    }

    pub fn new_from_env(db_client: Client) -> Self {
        Self::new(
            db_client,
            std::env::var("TABLE_NAME").expect("TABLE_NAME to be set in environment"),
        )
    }
}

#[async_trait::async_trait]
impl Repository for DynamoDBRepository {}
