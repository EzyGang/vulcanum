use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::services::github_app::service::webhooks::{
    is_completion_event, verify_signature, GithubWebhookError,
};

const SECRET: &str = "webhook-test-secret";
const CLOSED_PAYLOAD: &[u8] = br#"{
    "action":"closed",
    "number":42,
    "installation":{"id":123},
    "repository":{"full_name":"acme/widgets"}
}"#;
const OPENED_PAYLOAD: &[u8] = br#"{
    "action":"opened",
    "number":42,
    "installation":{"id":123},
    "repository":{"full_name":"acme/widgets"}
}"#;

#[test]
fn signature_accepts_exact_payload() {
    let signature = sign(CLOSED_PAYLOAD);

    assert!(verify_signature(Some(SECRET), &signature, CLOSED_PAYLOAD).is_ok());
}

#[test]
fn signature_rejects_modified_payload() {
    let signature = sign(CLOSED_PAYLOAD);

    assert!(matches!(
        verify_signature(Some(SECRET), &signature, b"modified"),
        Err(GithubWebhookError::InvalidSignature)
    ));
}

#[test]
fn signature_requires_configured_secret_and_sha256_format() {
    assert!(matches!(
        verify_signature(None, "sha256=00", CLOSED_PAYLOAD),
        Err(GithubWebhookError::NotConfigured)
    ));
    assert!(matches!(
        verify_signature(Some(SECRET), "sha1=00", CLOSED_PAYLOAD),
        Err(GithubWebhookError::InvalidSignature)
    ));
}

#[test]
fn only_closed_pull_request_events_trigger_completion() {
    assert!(is_completion_event("pull_request", CLOSED_PAYLOAD).expect("parse closed payload"));

    assert!(!is_completion_event("pull_request", OPENED_PAYLOAD).expect("parse opened payload"));
    assert!(!is_completion_event("push", b"not-json").expect("ignore unrelated event"));
}

fn sign(body: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(SECRET.as_bytes()).expect("valid HMAC key");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}
