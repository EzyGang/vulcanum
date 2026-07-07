use crate::providers::logging::redact_provider_output;

#[test]
fn redacts_secret_bearing_provider_output() {
    let cases = [
        "provider failed with OPENAI_API_KEY=sk-test-token",
        "provider failed with Authorization: Bearer sk-test-token",
        r#"provider failed with {"access_token":"sk-test-token"}"#,
    ];

    for line in cases {
        assert_eq!(redact_provider_output(line), "[redacted provider output]");
    }
}

#[test]
fn preserves_non_secret_provider_output() {
    let line = "provider failed with ordinary syntax error";

    assert_eq!(redact_provider_output(line), line);
}
