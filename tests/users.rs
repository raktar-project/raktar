mod common;

use common::setup::build_repository;
use raktar::repository::Repository;

#[tokio::test]
async fn test_users_are_not_recreated() {
    let repository = build_repository().await;

    let user_id = "test@raktar.io";
    let user = repository.get_or_create_user(user_id).await.unwrap();
    let user2 = repository.get_or_create_user(user_id).await.unwrap();

    assert_eq!(user, user2);
}

#[tokio::test]
async fn test_user_ids_are_incremented() {
    let repository = build_repository().await;

    let user = repository
        .get_or_create_user("test@raktar.io")
        .await
        .unwrap();
    let user2 = repository
        .get_or_create_user("test2@raktar.io")
        .await
        .unwrap();

    assert_eq!(user.id + 1, user2.id);
}
