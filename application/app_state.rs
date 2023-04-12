use aws_sdk_dynamodb::Client as DynamoDBClient;

use crate::storage::CrateStorage;

#[derive(Clone)]
pub struct AppState<S: CrateStorage> {
    pub db_client: DynamoDBClient,
    pub storage: S,
}
