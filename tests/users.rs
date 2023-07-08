mod common;

use raktar::models::user::CognitoUserData;
use raktar::repository::Repository;
use tracing_test::traced_test;

use common::setup::build_repository;

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
