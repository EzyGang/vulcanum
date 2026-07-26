use super::{
    hash_token, invite_redis_key, Duration, InMemoryTeamInviteStore, TeamInvitePayload,
    TeamInviteStore, Utc, Uuid,
};

#[tokio::test]
async fn invite_store_hashes_keys_and_consumes_once() {
    let store = InMemoryTeamInviteStore::new();
    let token = "raw-token-value";
    let payload = TeamInvitePayload {
        team_id: Uuid::new_v4(),
        created_by_user_id: "owner".to_owned(),
        role: "member".to_owned(),
        expires_at: Utc::now() + Duration::minutes(30),
    };

    store
        .save(token, &payload)
        .await
        .expect("invite payload should save");

    assert!(!store.contains_raw_key(token).await);
    assert_eq!(
        invite_redis_key(token),
        format!("vulcanum:team_invite:{}", hash_token(token))
    );
    assert_eq!(store.get(token).await.expect("get invite"), Some(payload));
    assert!(store
        .consume(token)
        .await
        .expect("consume invite")
        .is_some());
    assert!(store.consume(token).await.expect("consume again").is_none());
}

#[tokio::test]
async fn invite_store_hides_expired_payloads() {
    let store = InMemoryTeamInviteStore::new();
    let payload = TeamInvitePayload {
        team_id: Uuid::new_v4(),
        created_by_user_id: "owner".to_owned(),
        role: "member".to_owned(),
        expires_at: Utc::now() - Duration::minutes(1),
    };

    store
        .save("expired", &payload)
        .await
        .expect("invite payload should save");

    assert!(store.get("expired").await.expect("get expired").is_none());
    assert!(store
        .consume("expired")
        .await
        .expect("consume expired")
        .is_none());
}
