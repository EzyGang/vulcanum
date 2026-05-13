use sqlx::PgPool;

use crate::services::users::errors::UsersError;
use crate::services::users::repository::UsersRepository;

#[sqlx::test]
async fn find_or_create_user_creates_new_user(pool: PgPool) {
    let repo = UsersRepository::new();
    let email = "new@example.com";

    let user = repo
        .find_or_create_user(&pool, email)
        .await
        .expect("Should create user");

    assert_eq!(user.email, email);
    assert!(!user.id.is_empty());
    assert!(user.last_login_at.is_none());
}

#[sqlx::test]
async fn find_or_create_user_returns_existing_user(pool: PgPool) {
    let repo = UsersRepository::new();
    let email = "existing@example.com";

    let first = repo
        .find_or_create_user(&pool, email)
        .await
        .expect("Should create user");
    let second = repo
        .find_or_create_user(&pool, email)
        .await
        .expect("Should find user");

    assert_eq!(first.id, second.id);
    assert_eq!(first.created_at, second.created_at);
}

#[sqlx::test]
async fn find_user_by_id_returns_user(pool: PgPool) {
    let repo = UsersRepository::new();
    let email = "findme@example.com";

    let created = repo
        .find_or_create_user(&pool, email)
        .await
        .expect("Should create user");
    let found = repo
        .find_user_by_id(&pool, &created.id)
        .await
        .expect("Should find user");

    assert_eq!(found.id, created.id);
    assert_eq!(found.email, email);
}

#[sqlx::test]
async fn find_user_by_id_returns_not_found(pool: PgPool) {
    let repo = UsersRepository::new();

    let result = repo.find_user_by_id(&pool, "nonexistent-id").await;

    assert!(matches!(result, Err(UsersError::UserNotFound)));
}

#[sqlx::test]
async fn update_last_login_sets_timestamp(pool: PgPool) {
    let repo = UsersRepository::new();
    let email = "login@example.com";

    let user = repo
        .find_or_create_user(&pool, email)
        .await
        .expect("Should create user");

    assert!(user.last_login_at.is_none());

    repo.update_last_login(&pool, &user.id)
        .await
        .expect("Should update login");

    let updated = repo
        .find_user_by_id(&pool, &user.id)
        .await
        .expect("Should find user");

    assert!(updated.last_login_at.is_some());
}
