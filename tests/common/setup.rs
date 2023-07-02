use aws_sdk_dynamodb::types::{
    AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ProvisionedThroughput, ScalarAttributeType,
};
use aws_sdk_dynamodb::Client;
use raktar::repository::DynamoDBRepository;
use rand::distributions::{Alphanumeric, DistString};
use std::time::Duration;

fn generate_random_key() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
}

pub async fn build_repository() -> DynamoDBRepository {
    let table_name = generate_random_key();
    let access_key = generate_random_key();

    std::env::set_var("AWS_ACCESS_KEY_ID", &access_key);
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "test");

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
        .table_name(&table_name)
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

    wait_for_table(&db_client, &table_name).await;

    DynamoDBRepository::new(db_client, table_name)
}

async fn wait_for_table(db_client: &Client, table_name: &str) {
    for _ in 0..50 {
        let output = db_client
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .unwrap();

        let status = output.table().unwrap().table_status().unwrap();
        if *status == aws_sdk_dynamodb::types::TableStatus::Active {
            return;
        } else {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    panic!("dynamodb table never reached Active status");
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
