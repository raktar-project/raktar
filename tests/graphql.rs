use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use raktar::graphql::schema::build_schema;
use raktar::repository::{DynRepository, DynamoDBRepository};
use rand::distributions::{Alphanumeric, DistString};
use std::sync::Arc;

fn generate_random_table_name() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
}

async fn build_repository() -> DynamoDBRepository {
    let table_name = &generate_random_table_name();
    std::env::set_var("AWS_ACCESS_KEY_ID", "test");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");
    std::env::set_var("TABLE_NAME", table_name);

    let config = aws_config::from_env()
        .endpoint_url("http://localhost:8000")
        .region("eu-west-1")
        .load()
        .await;
    let db_client = Client::new(&config);
    let pk_schema = KeySchemaElement::builder()
        .key_type(KeyType::Hash)
        .attribute_name("pk".to_string())
        .build();
    let pk_definition = AttributeDefinition::builder()
        .attribute_name("pk")
        .attribute_type(ScalarAttributeType::S)
        .build();
    let sk_schema = KeySchemaElement::builder()
        .key_type(KeyType::Range)
        .attribute_name("sk".to_string())
        .build();
    let sk_definition = AttributeDefinition::builder()
        .attribute_name("sk")
        .attribute_type(ScalarAttributeType::S)
        .build();
    db_client
        .create_table()
        .table_name(table_name)
        .key_schema(pk_schema)
        .attribute_definitions(pk_definition)
        .key_schema(sk_schema)
        .attribute_definitions(sk_definition)
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5)
                .write_capacity_units(5)
                .build(),
        )
        .send()
        .await
        .expect("to be able to create table");
    DynamoDBRepository::new(db_client)
}

#[tokio::test]
async fn test_query() {
    let repository = Arc::new(build_repository().await) as DynRepository;
    let schema = build_schema(repository);

    let mutation = "
    mutation {
      generateToken(name: \"test token\") {
        id
        key
        token {
          id
          name
        }
      }
    }";
    let response = schema.execute(mutation).await;
    assert_eq!(response.errors.len(), 1);
}
