use uuid::Uuid;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::{
    authenticated_context, load_required_session, resolve_team, AppContext, AppRuntime,
};
use crate::tests::http::{FakeServer, Response};

const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

#[tokio::test]
async fn missing_login_fails_before_http() {
    let mut output = Vec::new();
    let mut load = || Ok(None);
    let mut save = |_: &AppSession| Ok(());
    let mut runtime = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    let error = load_required_session(&mut runtime).expect_err("missing login should fail");

    assert_eq!(error.to_string(), "Not logged in. Run `vulcanum login`.");
}

#[tokio::test]
async fn refresh_rotates_tokens_and_preserves_team_pin() {
    let server = FakeServer::start(vec![Response::ok(
        "POST",
        "/api/v1/auth/refresh",
        REFRESHED,
    )]);
    let team_id = Uuid::from_u128(42);
    let loaded = session(&server.url, Some(team_id));
    let mut saved = Vec::new();
    let mut output = Vec::new();
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |session: &AppSession| {
        saved.push(session.clone());
        Ok(())
    };
    let mut runtime = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    let context = authenticated_context(&mut runtime)
        .await
        .expect("refresh should succeed");
    let requests = server.finish();

    assert_eq!(requests[0].body, r#"{"refresh_token":"old-refresh"}"#);
    assert_eq!(context.session.access_token, "new-access");
    assert_eq!(context.session.refresh_token, "new-refresh");
    assert_eq!(context.session.team_id, Some(team_id));
    assert_eq!(saved, vec![context.session]);
}

#[tokio::test]
async fn fatal_refresh_returns_exact_expired_login_error() {
    for status in ["401 Unauthorized", "403 Forbidden"] {
        let server = FakeServer::start(vec![Response::status(
            "POST",
            "/api/v1/auth/refresh",
            status,
            r#"{"error":"access-secret refresh-secret"}"#,
        )]);
        let loaded = session(&server.url, None);
        let mut output = Vec::new();
        let mut load = || Ok(Some(loaded.clone()));
        let mut save = |_: &AppSession| Ok(());
        let mut runtime = AppRuntime {
            stdout: &mut output,
            load_session: &mut load,
            save_session: &mut save,
        };

        let error = authenticated_context(&mut runtime)
            .await
            .err()
            .expect("fatal refresh should fail");
        server.finish();

        assert_eq!(error.to_string(), "Login expired. Run `vulcanum login`.");
        assert!(!format!("{error:?}").contains("refresh-secret"));
    }
}

#[tokio::test]
async fn refresh_errors_are_sanitized_without_response_bodies() {
    let cases = [
        (
            "500 Internal Server Error",
            r#"{"error":"api-key=top-secret"}"#,
            "Refresh session failed: HTTP 500",
        ),
        (
            "200 OK",
            r#"{"access_token":"credential-value"}"#,
            "Refresh session failed: invalid response (HTTP 200)",
        ),
    ];

    for (status, body, expected) in cases {
        let server = FakeServer::start(vec![Response::status(
            "POST",
            "/api/v1/auth/refresh",
            status,
            body,
        )]);
        let loaded = session(&server.url, None);
        let mut output = Vec::new();
        let mut load = || Ok(Some(loaded.clone()));
        let mut save = |_: &AppSession| Ok(());
        let mut runtime = AppRuntime {
            stdout: &mut output,
            load_session: &mut load,
            save_session: &mut save,
        };

        let error = authenticated_context(&mut runtime)
            .await
            .err()
            .expect("refresh should fail");
        server.finish();

        assert_eq!(error.to_string(), expected);
        let debug = format!("{error:?}");
        assert!(!debug.contains("top-secret"));
        assert!(!debug.contains("credential-value"));
    }
}

#[tokio::test]
async fn transport_failure_keeps_only_operation_label() {
    let loaded = session("http://127.0.0.1:1", None);
    let mut output = Vec::new();
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut runtime = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    let error = authenticated_context(&mut runtime)
        .await
        .err()
        .expect("transport should fail");

    assert_eq!(error.to_string(), "Refresh session failed");
    assert_eq!(format!("{error:?}"), "Refresh session failed");
}

#[tokio::test]
async fn team_resolution_honors_override_pin_and_first_team_precedence() {
    let override_id = Uuid::from_u128(2);
    let pinned_id = Uuid::from_u128(3);
    let server = FakeServer::start(vec![Response::ok(
        "GET",
        &format!("/api/v1/teams/{override_id}"),
        team_json(override_id, "Override"),
    )]);
    let context = AppContext {
        client: ApiClient::new(&server.url),
        session: session(&server.url, Some(pinned_id)),
    };

    let team = resolve_team(&context, Some(override_id))
        .await
        .expect("override should resolve");
    server.finish();

    assert_eq!(team.id, override_id);
    assert_eq!(context.session.team_id, Some(pinned_id));

    let server = FakeServer::start(vec![Response::ok(
        "GET",
        &format!("/api/v1/teams/{pinned_id}"),
        team_json(pinned_id, "Pinned"),
    )]);
    let context = AppContext {
        client: ApiClient::new(&server.url),
        session: session(&server.url, Some(pinned_id)),
    };
    let team = resolve_team(&context, None)
        .await
        .expect("pin should resolve");
    server.finish();
    assert_eq!(team.id, pinned_id);

    let first_id = Uuid::from_u128(4);
    let body = format!("[{}]", team_json(first_id, "First"));
    let server = FakeServer::start(vec![Response::ok("GET", "/api/v1/teams", body)]);
    let context = AppContext {
        client: ApiClient::new(&server.url),
        session: session(&server.url, None),
    };
    let team = resolve_team(&context, None)
        .await
        .expect("first team should resolve");
    server.finish();
    assert_eq!(team.id, first_id);
}

#[tokio::test]
async fn inaccessible_pin_fails_closed_with_recovery_commands() {
    let pinned_id = Uuid::from_u128(5);
    let server = FakeServer::start(vec![Response::status(
        "GET",
        &format!("/api/v1/teams/{pinned_id}"),
        "404 Not Found",
        r#"{"error":"secret response"}"#,
    )]);
    let context = AppContext {
        client: ApiClient::new(&server.url),
        session: session(&server.url, Some(pinned_id)),
    };

    let error = resolve_team(&context, None)
        .await
        .expect_err("inaccessible pin should fail");
    server.finish();
    let message = error.to_string();

    assert!(message.contains(&pinned_id.to_string()));
    assert!(message.contains("vulcanum settings team set <UUID>"));
    assert!(message.contains("vulcanum settings team clear"));
    assert!(!format!("{error:?}").contains("secret response"));
}

fn team_json(id: Uuid, name: &str) -> String {
    format!(
        r#"{{"id":"{id}","name":"{name}","primary_model_provider_key":null,"primary_model_id":null,"small_model_provider_key":null,"small_model_id":null}}"#
    )
}

fn session(instance_url: &str, team_id: Option<Uuid>) -> AppSession {
    let mut session: AppSession = serde_json::from_value(serde_json::json!({
        "instance_url": instance_url,
        "access_token": "old-access",
        "refresh_token": "old-refresh",
        "refresh_expires_at": "2030-01-02T03:04:05Z",
    }))
    .expect("session should deserialize");
    session.team_id = team_id;
    session
}
