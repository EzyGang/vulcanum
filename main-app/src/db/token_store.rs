use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::RwLock;

struct TokenEntry {
    user_id: String,
    expires_at: DateTime<Utc>,
}

static STORE: std::sync::OnceLock<RwLock<HashMap<String, TokenEntry>>> = std::sync::OnceLock::new();

fn store() -> &'static RwLock<HashMap<String, TokenEntry>> {
    STORE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn insert_token(token: &str, user_id: &str, ttl_minutes: i64) {
    let entry = TokenEntry {
        user_id: user_id.to_owned(),
        expires_at: Utc::now() + Duration::minutes(ttl_minutes),
    };
    store()
        .write()
        .expect("token store lock poisoned")
        .insert(token.to_owned(), entry);
}

pub fn consume_token(token: &str) -> Option<String> {
    let mut map = store().write().expect("token store lock poisoned");
    let entry = map.remove(token)?;

    if Utc::now() > entry.expires_at {
        return None;
    }

    Some(entry.user_id)
}
