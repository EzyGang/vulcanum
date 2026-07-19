use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use super::super::configuration::{set_automation_with, set_columns_with, ColumnsOptions};
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const PROJECT_ID: Uuid = Uuid::from_u128(4);
const PROVIDER_ID: Uuid = Uuid::from_u128(3);
const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

#[tokio::test]
async fn automation_enable_and_disable_patch_only_enabled_state() {
    for enabled in [true, false] {
        let mut responses = project_responses(project_response(!enabled));
        responses.push(Response::ok(
            "PATCH",
            &project_target(),
            project_response(enabled),
        ));
        let server = FakeServer::start(responses);
        let mut output = Vec::new();
        let loaded = session(&server.url);
        let mut load = || Ok(Some(loaded.clone()));
        let mut save = ignore_save;
        let mut app = runtime(&mut output, &mut load, &mut save);

        set_automation_with(PROJECT_ID, enabled, None, &mut app)
            .await
            .expect("automation update should succeed");
        let requests = server.finish();
        let update = requests
            .iter()
            .find(|request| request.method == "PATCH")
            .expect("automation patch should be recorded");

        assert_eq!(update.target, project_target());
        assert_eq!(update.body, format!(r#"{{"enabled":{enabled}}}"#));
        assert!(String::from_utf8(output)
            .expect("output should be utf8")
            .contains(if enabled {
                "Automation enabled"
            } else {
                "Automation disabled"
            }));
    }
}

#[tokio::test]
async fn automation_enable_rejects_project_without_repositories_before_patch() {
    let responses = project_responses(project_response_with_repositories(false, "[]"));
    let server = FakeServer::start(responses);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    let error = set_automation_with(PROJECT_ID, true, None, &mut app)
        .await
        .expect_err("automation enable should require a repository");
    let requests = server.finish();

    assert!(error.to_string().contains(&format!(
        "vulcanum projects repos set {PROJECT_ID} --repo OWNER/NAME"
    )));
    assert!(!requests.iter().any(|request| request.method == "PATCH"));
}

#[tokio::test]
async fn column_marking_resolves_names_ids_and_slugs_before_atomic_patch() {
    let updated = r#"{"id":"00000000-0000-0000-0000-000000000004","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":true,"pickup_column":"to-do","progress_column":"in-progress","review_column":"in-review","done_column":"done","provider_id":"00000000-0000-0000-0000-000000000003","repo_full_names":["acme/api"]}"#;
    let mut responses = project_responses(project_response(true));
    responses.push(Response::ok("GET", &board_target(), board_response()));
    responses.push(Response::ok("PATCH", &project_target(), updated));
    let server = FakeServer::start(responses);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = ignore_save;
    let mut app = runtime(&mut output, &mut load, &mut save);

    set_columns_with(
        ColumnsOptions {
            project_id: PROJECT_ID,
            pickup: Some("To Do".to_owned()),
            in_progress: Some("col-progress".to_owned()),
            in_review: Some("in-review".to_owned()),
            done: Some("Done".to_owned()),
            team: None,
        },
        &mut app,
    )
    .await
    .expect("column update should succeed");
    let requests = server.finish();
    let update = requests
        .iter()
        .find(|request| request.method == "PATCH")
        .expect("column patch should be recorded");

    assert_eq!(
        update.body,
        r#"{"pickup_column":"to-do","progress_column":"in-progress","review_column":"in-review","done_column":"done"}"#
    );
    let output = String::from_utf8(output).expect("output should be utf8");
    for value in [
        "pickup=to-do",
        "in-progress=in-progress",
        "in-review=in-review",
        "done=done",
    ] {
        assert!(output.contains(value), "missing {value} in {output}");
    }
}

fn project_responses(project: String) -> Vec<Response> {
    vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok(
            "GET",
            &format!("/api/v1/teams/{TEAM_ID}"),
            r#"{"id":"00000000-0000-0000-0000-00000000002a","name":"Core","primary_model_provider_key":null,"primary_model_id":null,"small_model_provider_key":null,"small_model_id":null}"#,
        ),
        Response::ok("GET", &project_target(), project),
    ]
}

fn project_response(enabled: bool) -> String {
    project_response_with_repositories(enabled, r#"["acme/api"]"#)
}

fn project_response_with_repositories(enabled: bool, repo_full_names: &str) -> String {
    format!(
        r#"{{"id":"{PROJECT_ID}","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":{enabled},"pickup_column":"","progress_column":"","review_column":"","done_column":"","provider_id":"{PROVIDER_ID}","repo_full_names":{repo_full_names}}}"#
    )
}

fn board_response() -> &'static str {
    r#"{"provider_id":"00000000-0000-0000-0000-000000000003","provider_type":"kaneo","board":{"project":{"id":"KAN","name":"Platform","slug":"kan"},"columns":[{"id":"col-todo","name":"To Do","slug":"to-do","is_final":false,"tasks":[]},{"id":"col-progress","name":"In Progress","slug":"in-progress","is_final":false,"tasks":[]},{"id":"col-review","name":"In Review","slug":"in-review","is_final":false,"tasks":[]},{"id":"col-done","name":"Done","slug":"done","is_final":true,"tasks":[]}],"labels":[]},"task_augmentations":[]}"#
}

fn board_target() -> String {
    format!("/api/v1/task-board/providers/{PROVIDER_ID}/projects/KAN")
}

fn project_target() -> String {
    format!("/api/v1/projects/{PROJECT_ID}")
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

fn ignore_save(_: &AppSession) -> anyhow::Result<()> {
    Ok(())
}

fn runtime<'a>(
    output: &'a mut Vec<u8>,
    load: &'a mut dyn FnMut() -> anyhow::Result<Option<AppSession>>,
    save: &'a mut dyn FnMut(&AppSession) -> anyhow::Result<()>,
) -> AppRuntime<'a> {
    AppRuntime {
        stdout: output,
        load_session: load,
        save_session: save,
    }
}
