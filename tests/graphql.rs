use async_graphql::{Name, Request, Value};
use aws_sdk_dynamodb::types::{
    AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use raktar::graphql::handler::AuthenticatedUser;
use raktar::graphql::schema::build_schema;
use raktar::repository::{DynRepository, DynamoDBRepository};
use rand::distributions::{Alphanumeric, DistString};
use std::collections::HashSet;
use std::sync::Arc;

#[tokio::test]
async fn test_token_generation() {
    let repository = Arc::new(build_repository().await) as DynRepository;
    let schema = build_schema(repository);

    let mutation = r#"
    mutation {
      generateToken(name: "test token") {
        id
        key
        token {
          id
          name
        }
      }
    }"#;

    let response = schema.execute(build_request(mutation, 0)).await;
    assert_eq!(response.errors.len(), 0);

    let actual_name = extract_data(&response.data, &["generateToken", "token", "name"]);
    let expected_name = Value::String("test token".to_string());
    assert_eq!(actual_name, expected_name);

    let key = extract_data(&response.data, &["generateToken", "key"]);
    if let Value::String(k) = key {
        assert_eq!(k.len(), 32);
    } else {
        panic!("the key is not a string");
    }
}

#[tokio::test]
async fn test_my_tokens() {
    let repository = Arc::new(build_repository().await) as DynRepository;
    let schema = build_schema(repository);

    let mutation = r#"
    mutation {
      generateToken(name: "test token") {
        id
        key
        token {
          id
          name
        }
      }
    }"#;

    // We create a new token for user 0
    let response = schema.execute(build_request(mutation, 0)).await;
    assert_eq!(response.errors.len(), 0);

    // We create a new token with the same name for user 1
    let response = schema.execute(build_request(mutation, 1)).await;
    assert_eq!(response.errors.len(), 0);

    // For user 1, we create another token
    let mutation = r#"
    mutation {
      generateToken(name: "test token 2") {
        id
        key
        token {
          id
          name
        }
      }
    }"#;
    let response = schema.execute(build_request(mutation, 1)).await;
    assert_eq!(response.errors.len(), 0);

    // We get the tokens for user 1
    let query = r#"
    query {
      myTokens {
        name
      }
    }"#;
    let response = schema.execute(build_request(query, 1)).await;
    assert_eq!(response.errors.len(), 0);

    // There should be two tokens
    let tokens_data = extract_data(&response.data, &["myTokens"]);
    if let Value::List(tokens) = tokens_data {
        let actual: HashSet<String> = tokens
            .iter()
            .map(|t| extract_data(t, &["name"]).to_string())
            .collect();
        let mut expected = HashSet::new();
        expected.insert("\"test token\"".to_string());
        expected.insert("\"test token 2\"".to_string());
        assert_eq!(actual, expected);
    } else {
        panic!("tokens is not a list");
    }
}

fn extract_data(data: &Value, path: &[&str]) -> Value {
    let mut actual = data.clone();
    for p in path {
        if let Value::Object(mut obj) = actual {
            let key = Name::new(p);
            actual = obj.remove(&key).expect("key to exist");
        } else {
            panic!("value at {} is not an object", p);
        }
    }

    actual
}

fn generate_random_key() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
}

async fn build_repository() -> DynamoDBRepository {
    let table_name = &generate_random_key();
    let access_key = &generate_random_key();

    std::env::set_var("AWS_ACCESS_KEY_ID", access_key);
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
    let user_id_definition = AttributeDefinition::builder()
        .attribute_name("user_id")
        .attribute_type(ScalarAttributeType::N)
        .build();
    let gsi = build_user_data_gsi();
    db_client
        .create_table()
        .table_name(table_name)
        .key_schema(pk_schema)
        .attribute_definitions(pk_definition)
        .key_schema(sk_schema)
        .attribute_definitions(sk_definition)
        .attribute_definitions(user_id_definition)
        .global_secondary_indexes(gsi)
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

fn build_user_data_gsi() -> GlobalSecondaryIndex {
    let pk_schema = KeySchemaElement::builder()
        .key_type(KeyType::Hash)
        .attribute_name("user_id".to_string())
        .build();
    let sk_schema = KeySchemaElement::builder()
        .key_type(KeyType::Range)
        .attribute_name("pk".to_string())
        .build();

    GlobalSecondaryIndex::builder()
        .index_name("user_tokens")
        .key_schema(pk_schema)
        .key_schema(sk_schema)
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5)
                .write_capacity_units(5)
                .build(),
        )
        .projection(
            Projection::builder()
                .set_projection_type(Some(ProjectionType::All))
                .build(),
        )
        .build()
}

fn build_request(request_str: &str, user_id: u32) -> Request {
    let authenticated_user = AuthenticatedUser { id: user_id };
    let request: Request = request_str.into();
    request.data(authenticated_user)
}
