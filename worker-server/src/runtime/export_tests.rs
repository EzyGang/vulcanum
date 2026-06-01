use crate::runtime::export;

#[test]
fn parse_export_extracts_tokens() {
    let raw = r#"{
        "messages": [
            {
                "role": "user",
                "tokens": null,
                "model": null,
                "parts": []
            },
            {
                "role": "assistant",
                "tokens": {
                    "input": 500,
                    "output": 200,
                    "cache": {
                        "read": 50,
                        "write": 25
                    }
                },
                "model": {
                    "providerID": "anthropic",
                    "modelID": "claude-3"
                },
                "parts": [
                    {
                        "type": "text",
                        "text": "Done! Created PR at https://github.com/org/repo/pull/42"
                    }
                ]
            }
        ]
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.input_tokens, 500);
    assert_eq!(result.output_tokens, 200);
    assert_eq!(result.cache_read_tokens, 50);
    assert_eq!(result.cache_write_tokens, 25);
    assert_eq!(result.tokens_used, 775);
    assert_eq!(result.model_used.as_deref(), Some("anthropic/claude-3"));
    assert_eq!(
        result.pr_url.as_deref(),
        Some("https://github.com/org/repo/pull/42")
    );
}

#[test]
fn parse_export_handles_missing_cache() {
    let raw = r#"{
        "messages": [
            {
                "role": "assistant",
                "tokens": {
                    "input": 100,
                    "output": 50
                },
                "model": null,
                "parts": [
                    {
                        "type": "text",
                        "text": "result"
                    }
                ]
            }
        ]
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.input_tokens, 100);
    assert_eq!(result.output_tokens, 50);
    assert_eq!(result.cache_read_tokens, 0);
    assert_eq!(result.cache_write_tokens, 0);
}

#[test]
fn parse_export_handles_empty_messages() {
    let raw = r#"{"messages": []}"#;
    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.tokens_used, 0);
    assert!(result.model_used.is_none());
    assert!(result.pr_url.is_none());
}

#[test]
fn parse_export_extracts_pr_url_from_gitlab() {
    let raw = r#"{
        "messages": [
            {
                "role": "assistant",
                "tokens": {"input": 10, "output": 5},
                "model": null,
                "parts": [
                    {
                        "type": "text",
                        "text": "See https://gitlab.com/group/project/-/merge_requests/1"
                    }
                ]
            }
        ]
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(
        result.pr_url.as_deref(),
        Some("https://gitlab.com/group/project/-/merge_requests/1")
    );
}

#[test]
fn parse_export_model_with_only_model_id() {
    let raw = r#"{
        "messages": [
            {
                "role": "assistant",
                "tokens": {"input": 10, "output": 5},
                "model": {"modelID": "gpt-4"},
                "parts": []
            }
        ]
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.model_used.as_deref(), Some("gpt-4"));
}

#[test]
fn parse_export_invalid_json_returns_error() {
    let result = export::parse_export("not json");
    assert!(result.is_err());
}

#[test]
fn parse_export_no_pr_url_when_absent() {
    let raw = r#"{
        "messages": [
            {
                "role": "assistant",
                "tokens": {"input": 10, "output": 5},
                "model": null,
                "parts": [
                    {
                        "type": "text",
                        "text": "Task completed successfully"
                    }
                ]
            }
        ]
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert!(result.pr_url.is_none());
}
