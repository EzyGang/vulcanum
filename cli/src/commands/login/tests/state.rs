use super::support::{session, FakeServer, Fixture, Response, STATUS, TOKENS};

#[tokio::test]
async fn auth_failure_preserves_existing_session() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":true}"#),
        Response::status(
            "POST",
            "/api/v1/auth/instance-login",
            "401 Unauthorized",
            r#"{"error":"invalid password"}"#,
        ),
    ]);
    let existing = session("https://old.example");
    let mut fixture = Fixture::new(true);
    fixture.loaded = Some(existing.clone());
    fixture.prompt_instance = Some(server.url.clone());
    fixture.stdin = "wrong\n".to_owned();

    let error = fixture
        .run(None, true, None, false)
        .await
        .expect_err("invalid password should fail");

    server.finish();
    assert!(error.to_string().contains("invalid password"));
    assert!(fixture.saved.is_empty());
    assert_eq!(fixture.loaded, Some(existing));
}

#[tokio::test]
async fn explicit_instance_repairs_malformed_state_without_loading_it() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":true}"#),
        Response::ok("POST", "/api/v1/auth/instance-login", TOKENS),
    ]);
    let mut fixture = Fixture::new(false);
    fixture.load_error = true;
    fixture.stdin = "secret\n".to_owned();

    fixture
        .run(Some(server.url.clone()), true, None, false)
        .await
        .expect("explicit instance should skip malformed state");

    server.finish();
    assert_eq!(fixture.load_calls, 0);
    assert_eq!(fixture.saved.len(), 1);
}

#[tokio::test]
async fn malformed_state_aborts_implicit_instance_login() {
    let mut fixture = Fixture::new(true);
    fixture.load_error = true;

    let error = fixture
        .run(None, true, None, false)
        .await
        .expect_err("malformed state should abort");

    assert!(error
        .to_string()
        .contains("failed to parse state from C:\\broken\\app.json"));
    assert!(fixture.saved.is_empty());
}

#[tokio::test]
async fn persistence_failure_never_prints_success() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":true}"#),
        Response::ok("POST", "/api/v1/auth/instance-login", TOKENS),
    ]);
    let mut fixture = Fixture::new(false);
    fixture.stdin = "instance-secret\n".to_owned();
    fixture.save_error = true;

    fixture
        .run(Some(server.url.clone()), true, None, false)
        .await
        .expect_err("state persistence failure should abort");

    server.finish();
    assert!(!fixture.stderr_text().contains("Logged in to"));
}

#[tokio::test]
async fn implicit_instance_requires_terminal_before_loading_state() {
    let mut fixture = Fixture::new(false);
    fixture.load_error = true;

    let error = fixture
        .run(None, false, None, false)
        .await
        .expect_err("non-terminal implicit instance should fail");

    assert!(error.to_string().contains("pass --instance <URL>"));
    assert_eq!(fixture.load_calls, 0);
}
