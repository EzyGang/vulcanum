use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::AppRuntime;
use crate::tests::http::Response;

pub(super) const TEAM_ID: Uuid = Uuid::from_u128(42);
pub(super) const PROJECT_ID: Uuid = Uuid::from_u128(4);
pub(super) const PROVIDER_ID: Uuid = Uuid::from_u128(3);
pub(super) const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

pub(super) fn project_responses() -> Vec<Response> {
    vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &format!("/api/v1/teams/{TEAM_ID}"), team_response()),
        Response::ok(
            "GET",
            &format!("/api/v1/projects/{PROJECT_ID}"),
            project_response(),
        ),
    ]
}

pub(super) fn board_responses() -> Vec<Response> {
    let mut responses = project_responses();
    responses.push(Response::ok(
        "GET",
        &format!("/api/v1/task-board/providers/{PROVIDER_ID}/projects/KAN"),
        board_response(),
    ));
    responses
}

pub(super) fn task_response(title: &str, body: &str, status: &str) -> String {
    format!(
        r#"{{"task":{{"id":"task-1","title":"{title}","project_id":"KAN","description":"{body}","status":"{status}","priority":"medium","number":42,"project_slug":"kan","assignee_name":null,"created_at":"2030-01-02T03:04:05Z","updated_at":null,"labels":[{{"id":"label-1","name":"backend","color":"blue"}}]}}}}"#
    )
}

pub(super) fn session(instance_url: &str) -> AppSession {
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

pub(super) fn ignore_save(_: &AppSession) -> anyhow::Result<()> {
    Ok(())
}

pub(super) fn runtime<'a>(
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

fn team_response() -> &'static str {
    r#"{"id":"00000000-0000-0000-0000-00000000002a","name":"Core","primary_model_provider_key":null,"primary_model_id":null,"small_model_provider_key":null,"small_model_id":null}"#
}

fn project_response() -> String {
    format!(
        r#"{{"id":"{PROJECT_ID}","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":true,"pickup_column":"to-do","progress_column":"in-progress","review_column":"","done_column":"done","provider_id":"{PROVIDER_ID}","repo_full_names":[]}}"#
    )
}

fn board_response() -> &'static str {
    r#"{"provider_id":"00000000-0000-0000-0000-000000000003","provider_type":"kaneo","board":{"project":{"id":"KAN","name":"Platform","slug":"kan"},"columns":[{"id":"col-todo","name":"To Do","slug":"to-do","is_final":false,"tasks":[{"id":"task-1","title":"Fix parser","project_id":"KAN","description":"first body","status":"to-do","priority":"medium","number":42,"project_slug":"kan","assignee_name":null,"created_at":"2030-01-02T03:04:05Z","updated_at":null,"labels":[{"id":"label-1","name":"backend","color":"blue"}]},{"id":"task-2","title":"Write docs","project_id":"KAN","description":"second body","status":"to-do","priority":"low","number":43,"project_slug":"kan","assignee_name":"Dev","created_at":"2030-01-03T03:04:05Z","updated_at":null,"labels":[]}]},{"id":"col-progress","name":"In Progress","slug":"in-progress","is_final":false,"tasks":[{"id":"task-3","title":"Ship UI","project_id":"KAN","description":"frontend body","status":"in-progress","priority":"high","number":44,"project_slug":"kan","assignee_name":null,"created_at":"2030-01-04T03:04:05Z","updated_at":null,"labels":[{"id":"label-2","name":"frontend","color":"green"}]}]}],"labels":[{"id":"label-1","name":"backend","color":"blue"},{"id":"label-2","name":"frontend","color":"green"}]},"project_usage":{"total":{"tokens_used":0,"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_write_tokens":0,"finished_runs_count":0},"this_week":{"tokens_used":0,"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_write_tokens":0,"finished_runs_count":0}},"task_augmentations":[]}"#
}
