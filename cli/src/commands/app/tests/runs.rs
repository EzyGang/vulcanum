use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::runs::list_with;
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const RUN_ID: Uuid = Uuid::from_u128(99);
const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

#[tokio::test]
async fn work_run_list_renders_ticket_usage_and_status() {
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok(
            "GET",
            "/api/v1/runs",
            format!(
                r#"[{{"id":"{RUN_ID}","external_task_ref":"KAN-42","task_title":"Fix scheduler\nnow","status":"running","work_type":"implementation","tokens_used":1200,"input_tokens":700,"output_tokens":500,"cache_read_tokens":100,"cache_write_tokens":20,"model_used":"openai/gpt-5","duration_ms":4500,"created_at":"2030-01-02T03:04:05Z"}}]"#
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
        .expect("work run list should succeed");
    let requests = server.finish();
    let output = String::from_utf8(output).expect("output should be UTF-8");

    assert_eq!(
        requests[2].headers.get("x-team-id"),
        Some(&TEAM_ID.to_string())
    );
    assert!(output.starts_with(&format!("Work Runs — Core ({TEAM_ID})\n")));
    for value in [
        &RUN_ID.to_string(),
        "KAN-42",
        "Fix scheduler\\nnow",
        "implementation",
        "running",
        "1200 (700/500/120)",
        "openai/gpt-5",
        "4500 ms",
        "2030-01-02T03:04:05+00:00",
    ] {
        assert!(output.contains(value), "missing {value} in {output}");
    }
}

#[tokio::test]
async fn empty_work_run_list_prints_exact_message() {
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok("GET", "/api/v1/runs", "[]"),
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
        .expect("empty work run list should succeed");
    server.finish();

    assert_eq!(
        String::from_utf8(output).expect("output should be UTF-8"),
        format!("No work runs found for team Core ({TEAM_ID}).\n")
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
