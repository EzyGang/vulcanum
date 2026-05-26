use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};

struct TokenEntry {
    user_id: String,
    expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TokenStore {
    inner: Arc<RwLock<HashMap<String, TokenEntry>>>,
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, token: &str, user_id: &str, ttl_minutes: i64) {
        let entry = TokenEntry {
            user_id: user_id.to_owned(),
            expires_at: Utc::now() + Duration::minutes(ttl_minutes),
        };
        let mut map = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let now = Utc::now();
        map.retain(|_, v| v.expires_at > now);
        map.insert(token.to_owned(), entry);
    }

    #[must_use]
    pub fn consume(&self, token: &str) -> Option<String> {
        let mut map = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let entry = map.remove(token)?;

        if Utc::now() > entry.expires_at {
            return None;
        }

        Some(entry.user_id)
    }
}
