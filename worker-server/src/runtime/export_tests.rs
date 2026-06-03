use crate::runtime::export;

#[test]
fn parse_export_extracts_tokens_and_model() {
    let raw = r#"{
        "info": {
            "tokens": {
                "input": 500,
                "output": 200,
                "cache": {
                    "read": 50,
                    "write": 25
                }
            },
            "model": {
                "id": "claude-3",
                "providerID": "anthropic"
            }
        }
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.input_tokens, 500);
    assert_eq!(result.output_tokens, 200);
    assert_eq!(result.cache_read_tokens, 50);
    assert_eq!(result.cache_write_tokens, 25);
    assert_eq!(result.tokens_used, 775);
    assert_eq!(result.model_used.as_deref(), Some("anthropic/claude-3"));
}

#[test]
fn parse_export_handles_missing_cache() {
    let raw = r#"{
        "info": {
            "tokens": {
                "input": 100,
                "output": 50
            }
        }
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.input_tokens, 100);
    assert_eq!(result.output_tokens, 50);
    assert_eq!(result.cache_read_tokens, 0);
    assert_eq!(result.cache_write_tokens, 0);
}

#[test]
fn parse_export_handles_null_tokens() {
    let raw = r#"{
        "info": {
            "tokens": {
                "input": null,
                "output": null
            }
        }
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.tokens_used, 0);
    assert!(result.model_used.is_none());
}

#[test]
fn parse_export_model_with_only_id() {
    let raw = r#"{
        "info": {
            "tokens": {
                "input": 10,
                "output": 5
            },
            "model": {
                "id": "gpt-4"
            }
        }
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert_eq!(result.model_used.as_deref(), Some("gpt-4"));
}

#[test]
fn parse_export_model_missing() {
    let raw = r#"{
        "info": {
            "tokens": {
                "input": 10,
                "output": 5
            }
        }
    }"#;

    let result = export::parse_export(raw).unwrap();
    assert!(result.model_used.is_none());
}

#[test]
fn parse_export_invalid_json_returns_error() {
    let result = export::parse_export("not json");
    assert!(result.is_err());
}
