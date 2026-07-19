use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use super::super::repos::{list_with as list_repos_with, set_with, EditOptions};
use super::super::runtime::ProjectsRuntime;
use super::super::{add_with, list_with, AddOptions};
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const PROVIDER_ID: Uuid = Uuid::from_u128(3);
const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

#[tokio::test]
async fn project_list_renders_configured_source_and_repositories() {
    let project_id = Uuid::from_u128(4);
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok(
            "GET",
            "/api/v1/projects",
            format!(
                r#"[{{"id":"{project_id}","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":true,"provider_id":"{PROVIDER_ID}","repo_full_names":["acme/api"]}}]"#
            ),
        ),
        Response::ok("GET", "/api/v1/providers", provider_response()),
    ]);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut app = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    list_with(None, &mut app)
        .await
        .expect("project list should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    for value in [
        "Projects — Core",
        &project_id.to_string(),
        "Platform",
        "Linear",
        "KAN",
        "enabled",
        "acme/api",
    ] {
        assert!(output.contains(value), "missing {value} in {output}");
    }
}

#[tokio::test]
async fn interactive_project_add_selects_catalog_project_and_repository() {
    let project_id = Uuid::from_u128(4);
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok("GET", "/api/v1/projects", "[]"),
        Response::ok("GET", "/api/v1/providers", provider_response()),
        Response::ok(
            "GET",
            &format!("/api/v1/providers/{PROVIDER_ID}/workspaces"),
            r#"[{"id":"core","name":"Core"}]"#,
        ),
        Response::ok(
            "GET",
            &format!("/api/v1/providers/{PROVIDER_ID}/projects?workspace_id=core"),
            r#"[{"id":"KAN","name":"Platform","slug":"platform"}]"#,
        ),
        Response::ok(
            "GET",
            "/api/v1/github/installation",
            r#"{"id":9,"account_login":"acme"}"#,
        ),
        Response::ok(
            "GET",
            "/api/v1/github/repos",
            r#"[{"owner":"acme","name":"api","full_name":"acme/api"}]"#,
        ),
        Response::ok(
            "POST",
            "/api/v1/projects",
            format!(
                r#"{{"id":"{project_id}","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":false,"provider_id":"{PROVIDER_ID}","repo_full_names":["acme/api"]}}"#
            ),
        ),
    ]);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut app = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };
    let mut projects = ProjectsRuntime {
        stdin_is_terminal: true,
        select: Box::new(|_, _| Ok(0)),
        select_many: Box::new(|_, _, _| Ok(vec![0])),
    };

    add_with(
        AddOptions {
            provider: None,
            workspace: None,
            project: None,
            repos: Vec::new(),
            team: None,
        },
        &mut app,
        &mut projects,
    )
    .await
    .expect("project add should succeed");
    let requests = server.finish();
    let create = requests
        .iter()
        .find(|request| request.method == "POST" && request.target == "/api/v1/projects")
        .expect("create request should be recorded");

    assert!(create.body.contains(r#""external_project_id":"KAN""#));
    assert!(create.body.contains(r#""repo_full_names":["acme/api"]"#));
    assert_eq!(create.headers.get("x-team-id"), Some(&TEAM_ID.to_string()));
    assert_eq!(
        String::from_utf8(output).expect("output should be utf8"),
        format!(
            "Added project Platform ({project_id}) for team {TEAM_ID} with automation disabled and 1 attached repositories.\n"
        )
    );
}

#[tokio::test]
async fn repository_list_pulls_and_sorts_available_github_repositories() {
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok(
            "GET",
            "/api/v1/github/installation",
            r#"{"id":9,"account_login":"acme"}"#,
        ),
        Response::ok(
            "GET",
            "/api/v1/github/repos",
            r#"[{"owner":"acme","name":"web","full_name":"acme/web"},{"owner":"acme","name":"api","full_name":"acme/api"}]"#,
        ),
    ]);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut app = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };

    list_repos_with(None, &mut app)
        .await
        .expect("repository list should succeed");
    server.finish();
    let output = String::from_utf8(output).expect("output should be utf8");

    assert!(output.starts_with(&format!(
        "Available GitHub repositories — Core ({TEAM_ID})\n"
    )));
    for heading in ["OWNER", "NAME", "FULL NAME"] {
        assert!(output.contains(heading), "missing {heading} in {output}");
    }
    let api = output
        .find("acme/api")
        .expect("API repository should render");
    let web = output
        .find("acme/web")
        .expect("web repository should render");
    assert!(api < web, "repositories should be sorted in {output}");
}

#[tokio::test]
async fn interactive_repository_edit_pulls_available_repos_and_preselects_attached_ones() {
    let project_id = Uuid::from_u128(4);
    let current = format!(
        r#"{{"id":"{project_id}","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":false,"provider_id":"{PROVIDER_ID}","repo_full_names":["acme/api"]}}"#
    );
    let updated = format!(
        r#"{{"id":"{project_id}","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":false,"provider_id":"{PROVIDER_ID}","repo_full_names":["acme/web"]}}"#
    );
    let project_target = format!("/api/v1/projects/{project_id}");
    let server = FakeServer::start(vec![
        Response::ok("POST", "/api/v1/auth/refresh", REFRESHED),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok("GET", &project_target, current),
        Response::ok(
            "GET",
            "/api/v1/github/installation",
            r#"{"id":9,"account_login":"acme"}"#,
        ),
        Response::ok(
            "GET",
            "/api/v1/github/repos",
            r#"[{"owner":"acme","name":"web","full_name":"acme/web"},{"owner":"acme","name":"api","full_name":"acme/api"}]"#,
        ),
        Response::ok("PATCH", &project_target, updated),
    ]);
    let mut output = Vec::new();
    let loaded = session(&server.url);
    let mut load = || Ok(Some(loaded.clone()));
    let mut save = |_: &AppSession| Ok(());
    let mut app = AppRuntime {
        stdout: &mut output,
        load_session: &mut load,
        save_session: &mut save,
    };
    let mut projects = ProjectsRuntime {
        stdin_is_terminal: true,
        select: Box::new(|_, _| Ok(0)),
        select_many: Box::new(|_, labels, defaults| {
            assert_eq!(labels, ["acme/api", "acme/web"]);
            assert_eq!(defaults, [true, false]);
            Ok(vec![1])
        }),
    };

    set_with(
        EditOptions {
            project_id,
            repos: Vec::new(),
            clear: false,
            team: None,
        },
        &mut app,
        &mut projects,
    )
    .await
    .expect("repository edit should succeed");
    let requests = server.finish();
    let patch = requests
        .iter()
        .find(|request| request.method == "PATCH" && request.target == project_target)
        .expect("update request should be recorded");

    assert_eq!(patch.body, r#"{"repo_full_names":["acme/web"]}"#);
    assert_eq!(
        String::from_utf8(output).expect("output should be utf8"),
        format!("Updated project Platform ({project_id}) with 1 attached repositories.\n")
    );
}

fn provider_response() -> String {
    format!(
        r#"[{{"id":"{PROVIDER_ID}","name":"Linear","provider_type":"linear","instance_url":"https://linear.app"}}]"#
    )
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
