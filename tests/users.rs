mod common;

use raktar::models::user::{CognitoUserData, User};
use raktar::repository::UserRepository;
use tracing_test::traced_test;

use crate::common::setup::create_db_client;
use common::setup::build_repository;
use raktar::repository::dynamodb::user::put_user;

#[tokio::test]
#[traced_test]
async fn test_users_are_not_recreated() {
    let repository = build_repository().await;

    let user_data = CognitoUserData {
        login: "test@raktar.io".to_string(),
        given_name: "Bruce".to_string(),
        family_name: "Wayne".to_string(),
    };
    let user = repository
        .update_or_create_user(user_data.clone())
        .await
        .unwrap();
    let user2 = repository.update_or_create_user(user_data).await.unwrap();

    assert_eq!(user, user2);
}

#[tokio::test]
#[traced_test]
async fn test_user_ids_are_incremented() {
    let repository = build_repository().await;

    let user_data_1 = CognitoUserData {
        login: "test@raktar.io".to_string(),
        given_name: "Bruce".to_string(),
        family_name: "Wayne".to_string(),
    };
    let user1 = repository.update_or_create_user(user_data_1).await.unwrap();

    let user_data_2 = CognitoUserData {
        login: "test2@raktar.io".to_string(),
        given_name: "Clark".to_string(),
        family_name: "Kent".to_string(),
    };
    let user2 = repository.update_or_create_user(user_data_2).await.unwrap();

    assert_eq!(user1.id, 1);
    assert_eq!(user2.id, 2);
}

#[tokio::test]
async fn test_cant_put_the_same_new_user_twice() {
    let (db_client, table_name) = create_db_client().await;

    let user = User {
        id: 33,
        login: "user_x@raktar.io".to_string(),
        given_name: "Bruce".to_string(),
        family_name: "Wayne".to_string(),
    };

    let result = put_user(&db_client, &table_name, user.clone(), true).await;
    assert!(result.is_ok());

    let result = put_user(&db_client, &table_name, user.clone(), true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_updating_user_works() {
    let (db_client, table_name) = create_db_client().await;

    let user = User {
        id: 33,
        login: "user_x@raktar.io".to_string(),
        given_name: "Bruce".to_string(),
        family_name: "Wayne".to_string(),
    };

    let result = put_user(&db_client, &table_name, user.clone(), false).await;
    assert!(result.is_ok());

    let result = put_user(&db_client, &table_name, user.clone(), false).await;
    assert!(result.is_ok());
}
