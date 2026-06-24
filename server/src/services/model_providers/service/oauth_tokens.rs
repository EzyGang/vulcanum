use base64::Engine;
use chrono::Utc;

const DEFAULT_TOKEN_EXPIRES_SECONDS: i64 = 60 * 60 * 24;

#[must_use]
pub(crate) fn oauth_expires_at(expires_in: Option<i64>) -> chrono::DateTime<Utc> {
    Utc::now() + chrono::Duration::seconds(expires_in.unwrap_or(DEFAULT_TOKEN_EXPIRES_SECONDS))
}

#[must_use]
pub(crate) fn extract_account_id(token: &str) -> Option<String> {
    jwt_payload(token).and_then(|payload| {
        payload
            .get("chatgpt_account_id")
            .or_else(|| payload.get("account_id"))
            .and_then(|value| value.as_str())
            .map(str::to_owned)
    })
}

#[must_use]
pub(crate) fn extract_email(token: &str) -> Option<String> {
    jwt_payload(token).and_then(|payload| {
        payload
            .get("email")
            .and_then(|value| value.as_str())
            .map(str::to_owned)
    })
}

#[must_use]
fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    serde_json::from_slice(&decoded).ok()
}
