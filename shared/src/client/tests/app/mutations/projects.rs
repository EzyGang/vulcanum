use uuid::Uuid;

use crate::api::app::projects::{CreateProjectRequest, UpdateProjectRequest};
use crate::client::tests::app::{assert_header, TEAM_ID};
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

const PROVIDER_ID: Uuid = Uuid::from_u128(3);
const PROJECT_RESPONSE: &str = r#"{"id":"00000000-0000-0000-0000-000000000004","external_project_id":"KAN","name":"Platform","external_workspace_id":"core","enabled":false,"provider_id":"00000000-0000-0000-0000-000000000003","repo_full_names":["acme/api"]}"#;

#[tokio::test]
async fn list_projects_gets_team_scoped_projects() {
    let (base_url, handle) = serve_once("200 OK", format!("[{PROJECT_RESPONSE}]"));

    let projects = ApiClient::new(base_url)
        .list_projects(TEAM_ID, "app-access")
        .await
        .expect("projects should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with("GET /api/v1/projects "));
    assert_header(&request, "authorization", "Bearer app-access");
    assert_header(&request, "x-team-id", &TEAM_ID.to_string());
    assert_eq!(projects[0].repo_full_names, ["acme/api"]);
}

#[tokio::test]
async fn provider_project_catalog_sends_workspace_query() {
    let response = r#"[{"id":"KAN","name":"Platform","slug":"platform"}]"#;
    let (base_url, handle) = serve_once("200 OK", response);

    let projects = ApiClient::new(base_url)
        .list_provider_projects(TEAM_ID, PROVIDER_ID, "workspace-1", "app-access")
        .await
        .expect("provider projects should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with(&format!(
        "GET /api/v1/providers/{PROVIDER_ID}/projects?workspace_id=workspace-1 "
    )));
    assert_header(&request, "x-team-id", &TEAM_ID.to_string());
    assert_eq!(projects[0].id, "KAN");
}

#[tokio::test]
async fn create_project_posts_selected_source_and_repositories() {
    let (base_url, handle) = serve_once("200 OK", PROJECT_RESPONSE);
    let request = CreateProjectRequest {
        external_project_id: "KAN".to_owned(),
        external_workspace_id: "core".to_owned(),
        name: "Platform".to_owned(),
        provider_id: PROVIDER_ID,
        enabled: false,
        repo_full_names: vec!["acme/api".to_owned()],
    };

    let created = ApiClient::new(base_url)
        .create_project(TEAM_ID, &request, "app-access")
        .await
        .expect("project should be created");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with("POST /api/v1/projects "));
    assert_header(&wire, "authorization", "Bearer app-access");
    assert!(wire.ends_with(
        r#"{"external_project_id":"KAN","external_workspace_id":"core","name":"Platform","provider_id":"00000000-0000-0000-0000-000000000003","enabled":false,"repo_full_names":["acme/api"]}"#
    ));
    assert_eq!(created.id, Uuid::from_u128(4));
}

#[tokio::test]
async fn update_project_patches_repositories_only() {
    let (base_url, handle) = serve_once("200 OK", PROJECT_RESPONSE);
    let request = UpdateProjectRequest {
        repo_full_names: Some(vec!["acme/api".to_owned()]),
        ..UpdateProjectRequest::default()
    };

    let updated = ApiClient::new(base_url)
        .update_project(TEAM_ID, Uuid::from_u128(4), &request, "app-access")
        .await
        .expect("project should update");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with("PATCH /api/v1/projects/00000000-0000-0000-0000-000000000004 "));
    assert_header(&wire, "x-team-id", &TEAM_ID.to_string());
    assert!(wire.ends_with(r#"{"repo_full_names":["acme/api"]}"#));
    assert_eq!(updated.repo_full_names, ["acme/api"]);
}

#[tokio::test]
async fn github_repo_catalog_gets_team_scoped_repositories() {
    let response = r#"[{"owner":"acme","name":"api","full_name":"acme/api"}]"#;
    let (base_url, handle) = serve_once("200 OK", response);

    let repos = ApiClient::new(base_url)
        .list_github_repos(TEAM_ID, "app-access")
        .await
        .expect("repositories should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with("GET /api/v1/github/repos "));
    assert_header(&request, "x-team-id", &TEAM_ID.to_string());
    assert_eq!(repos[0].full_name, "acme/api");
}
