use serde_json::json;

use crate::api::wire::SubmitResultRequest;
use crate::runtime::types::FinishStatus;

#[test]
fn submit_result_keeps_blocked_reason_separate_from_summary() {
    let request: SubmitResultRequest = serde_json::from_value(json!({
        "pr_urls": [],
        "exit_code": 1,
        "tokens_used": 0,
        "duration_ms": 0,
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
        "model_used": null,
        "finish_status": "blocked",
        "result_summary": "Partial outcome",
        "blocked_reason": "Waiting for repository access"
    }))
    .expect("result request should deserialize");

    assert_eq!(request.finish_status, Some(FinishStatus::Blocked));
    assert_eq!(request.result_summary.as_deref(), Some("Partial outcome"));
    assert_eq!(
        request.blocked_reason.as_deref(),
        Some("Waiting for repository access")
    );
}

#[test]
fn submit_result_defaults_missing_blocked_reason_to_none() {
    let request: SubmitResultRequest = serde_json::from_value(json!({
        "pr_urls": [],
        "exit_code": 0,
        "tokens_used": 0,
        "duration_ms": 0,
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
        "model_used": null,
        "finish_status": "completed",
        "result_summary": null
    }))
    .expect("result request should deserialize");

    assert!(request.blocked_reason.is_none());
}
