use vulcanum_shared::constants::DEFAULT_TEAM_ID;

use super::support::{FakeServer, Fixture, Response, STATUS, TOKENS};

#[tokio::test]
async fn password_stdin_saves_canonical_session_verbatim() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":true}"#),
        Response::ok("POST", "/api/v1/auth/instance-login", TOKENS),
    ]);
    let server_url = server.url.clone();
    let mut fixture = Fixture::new(false);
    fixture.stdin = "  instance-secret  \r\n".to_owned();

    let result = fixture
        .run(Some(format!("{}/", server.url)), true, None, false)
        .await;

    result.expect("single-user login should succeed");
    let requests = server.finish();
    assert_eq!(requests[0].target, "/api/v1/status");
    assert_eq!(requests[1].target, "/api/v1/auth/mode");
    assert_eq!(requests[2].body, r#"{"password":"  instance-secret  "}"#);
    fixture.assert_saved_session(&server_url, Some(DEFAULT_TEAM_ID));
    fixture.assert_secrets_not_emitted(&["instance-secret", "access-secret", "refresh-secret"]);
    assert!(fixture
        .stderr_text()
        .contains(&format!("Logged in to {server_url}")));
}

#[tokio::test]
async fn empty_and_unavailable_password_inputs_fail_without_authentication() {
    for (terminal, password_stdin, stdin, prompt_password) in [
        (false, true, "\n", ""),
        (false, false, "", ""),
        (true, false, "", ""),
    ] {
        let server = FakeServer::start(vec![
            Response::ok("GET", "/api/v1/status", STATUS),
            Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":true}"#),
        ]);
        let mut fixture = Fixture::new(terminal);
        fixture.stdin = stdin.to_owned();
        fixture.prompt_password = prompt_password.to_owned();

        let result = fixture
            .run(Some(server.url.clone()), password_stdin, None, false)
            .await;

        server.finish();
        assert!(result.is_err());
        assert!(fixture.saved.is_empty());
    }
}
