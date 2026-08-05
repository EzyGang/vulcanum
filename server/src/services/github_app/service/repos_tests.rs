use super::repos::installation_token_request_body;

#[test]
fn installation_token_request_includes_workflow_write_permission() {
    let body = installation_token_request_body(vec!["vulcanum".to_owned()]);

    assert_eq!(
        body,
        serde_json::json!({
            "repositories": ["vulcanum"],
            "permissions": {
                "contents": "write",
                "pull_requests": "write",
                "workflows": "write"
            }
        })
    );
}
