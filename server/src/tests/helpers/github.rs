use super::{Hmac, Mac, Sha256, GITHUB_WEBHOOK_SECRET};

pub fn github_webhook_payload(action: &str) -> Vec<u8> {
    github_pull_request_payload(action, true)
}

pub fn github_pull_request_payload(action: &str, merged: bool) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "action": action,
        "number": 42,
        "installation": {"id": 123},
        "repository": {"full_name": "acme/widgets"},
        "pull_request": {"merged": merged},
    }))
    .expect("serialize GitHub webhook fixture")
}

pub fn sign_github_webhook(body: &[u8]) -> String {
    let mut mac =
        Hmac::<Sha256>::new_from_slice(GITHUB_WEBHOOK_SECRET.as_bytes()).expect("valid HMAC key");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}
