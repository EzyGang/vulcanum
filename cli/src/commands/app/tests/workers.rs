use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::workers::list_with;
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

#[tokio::test]
async fn worker_list_renders_worker_snapshot_in_api_order() {
    let worker_id = Uuid::from_u128(7);
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok(
            "GET",
            "/api/v1/workers",
            format!(
                r#"[{{"id":"{worker_id}","name":"build-a","status":"busy","last_seen":null,"active_jobs":1,"max_concurrent_jobs":3}}]"#
            ),
        ),
    ]);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut runtime = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    list_with(None, &mut runtime)
        .await
        .expect("worker list should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    assert!(output.starts_with(&format!("Workers — Core ({TEAM_ID})\n")));
    for value in [
        "ID",
        "NAME",
        "STATUS",
        "LAST SEEN",
        "LOAD",
        "build-a",
        "busy",
        "never",
        "1/3",
    ] {
        assert!(output.contains(value), "missing {value} in {output}");
    }
}

#[tokio::test]
async fn empty_worker_list_prints_exact_message() {
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok("GET", "/api/v1/workers", "[]"),
    ]);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut runtime = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    list_with(None, &mut runtime)
        .await
        .expect("empty list should succeed");
    server.finish();

    assert_eq!(
        String::from_utf8(output).expect("output should be utf8"),
        format!("No workers found for team Core ({TEAM_ID}).\n")
    );
}

fn team_target() -> String {
    format!("/api/v1/teams/{TEAM_ID}")
}

fn team_response() -> &'static str {
    r#"{"id":"00000000-0000-0000-0000-00000000002a","name":"Core","primary_model_provider_key":null,"primary_model_id":null,"small_model_provider_key":null,"small_model_id":null}"#
}

fn session(instance_url: &str) -> AppSession {
    let mut session: AppSession = serde_json::from_value(serde_json::json!({
        "instance_url": instance_url,
        "access_token": "old-access",
        "refresh_token": "old-refresh",
        "refresh_expires_at": "2030-01-02T03:04:05Z",
    }))
    .expect("session should deserialize");
    session.team_id = Some(TEAM_ID);
    session
}
