use uuid::Uuid;

use super::support::{session, FakeServer, Fixture, Response, STATUS, TOKENS};

#[tokio::test]
async fn auth_code_saves_same_session_schema() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":false}"#),
        Response::ok("POST", "/api/v1/auth/exchange", TOKENS),
    ]);
    let server_url = server.url.clone();
    let mut fixture = Fixture::new(false);

    fixture
        .run(
            Some(server.url.clone()),
            false,
            Some(" one-time-code ".to_owned()),
            false,
        )
        .await
        .expect("multi-user login should succeed");

    let requests = server.finish();
    assert_eq!(requests[2].target, "/api/v1/auth/exchange");
    assert_eq!(requests[2].body, r#"{"code":"one-time-code"}"#);
    fixture.assert_saved_session(&server_url, None);
    fixture.assert_secrets_not_emitted(&["one-time-code", "access-secret", "refresh-secret"]);
    assert!(fixture.opened.is_empty());
}

#[tokio::test]
async fn non_terminal_handoff_prints_url_warning_and_rerun_command() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":false}"#),
    ]);
    let server_url = server.url.clone();
    let mut fixture = Fixture::new(false);
    fixture.open_error = true;

    fixture
        .run(Some(server.url.clone()), false, None, false)
        .await
        .expect_err("non-terminal handoff should stop after printing instructions");

    server.finish();
    let expected_url = format!("{server_url}/api/v1/auth/github/start?return_to=%2Fcli-login");
    assert_eq!(fixture.stdout_text(), format!("{expected_url}\n"));
    let stderr = fixture.stderr_text();
    assert!(stderr.contains("[WARNING] Could not open the default browser"));
    assert!(stderr.contains(&format!(
        "vulcanum login --instance {server_url} --auth-code <CODE>"
    )));
    assert!(fixture.saved.is_empty());
}

#[tokio::test]
async fn browser_failure_is_non_fatal_when_terminal_code_is_available() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":false}"#),
        Response::ok("POST", "/api/v1/auth/exchange", TOKENS),
    ]);
    let mut fixture = Fixture::new(true);
    fixture.open_error = true;
    fixture.prompt_code = "one-time-code".to_owned();

    fixture
        .run(Some(server.url.clone()), false, None, false)
        .await
        .expect("browser failure should not prevent code exchange");

    server.finish();
    assert!(fixture.stderr_text().contains("[WARNING]"));
    assert_eq!(fixture.saved.len(), 1);
}

#[tokio::test]
async fn no_browser_skips_launch_and_exchanges_prompted_code() {
    let server = FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":false}"#),
        Response::ok("POST", "/api/v1/auth/exchange", TOKENS),
    ]);
    let mut fixture = Fixture::new(true);
    fixture.prompt_code = "one-time-code".to_owned();

    fixture
        .run(Some(server.url.clone()), false, None, true)
        .await
        .expect("--no-browser login should exchange the prompted code");

    server.finish();
    assert!(fixture.opened.is_empty());
    assert_eq!(fixture.saved.len(), 1);
}

#[tokio::test]
async fn implicit_same_instance_relogin_preserves_team_pin() {
    let server = multi_user_server();
    let team_id = Uuid::from_u128(42);
    let mut loaded = session(&server.url);
    loaded.team_id = Some(team_id);
    let mut fixture = Fixture::new(true);
    fixture.loaded = Some(loaded);
    fixture.prompt_instance = Some(server.url.clone());

    fixture
        .run(None, false, Some("code".to_owned()), false)
        .await
        .expect("implicit relogin should succeed");

    let server_url = server.url.clone();
    server.finish();
    fixture.assert_saved_session(&server_url, Some(team_id));
}

#[tokio::test]
async fn different_or_explicit_multi_user_login_clears_team_pin() {
    let server = multi_user_server();
    let mut loaded = session("https://old.example");
    loaded.team_id = Some(Uuid::from_u128(42));
    let mut fixture = Fixture::new(true);
    fixture.loaded = Some(loaded);
    fixture.prompt_instance = Some(server.url.clone());

    fixture
        .run(None, false, Some("code".to_owned()), false)
        .await
        .expect("different-instance login should succeed");

    let server_url = server.url.clone();
    server.finish();
    fixture.assert_saved_session(&server_url, None);

    let server = multi_user_server();
    let mut fixture = Fixture::new(true);
    fixture.load_error = true;
    fixture
        .run(
            Some(server.url.clone()),
            false,
            Some("code".to_owned()),
            false,
        )
        .await
        .expect("explicit login should skip and replace prior state");
    let server_url = server.url.clone();
    server.finish();
    assert_eq!(fixture.load_calls, 0);
    fixture.assert_saved_session(&server_url, None);
}

fn multi_user_server() -> FakeServer {
    FakeServer::start(vec![
        Response::ok("GET", "/api/v1/status", STATUS),
        Response::ok("GET", "/api/v1/auth/mode", r#"{"is_single_user":false}"#),
        Response::ok("POST", "/api/v1/auth/exchange", TOKENS),
    ])
}
