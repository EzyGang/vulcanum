use uuid::Uuid;

use crate::api::app::task_board::CreateTaskRequest;
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

const TEAM_ID: Uuid = Uuid::from_u128(1);
const PROVIDER_ID: Uuid = Uuid::from_u128(3);

#[tokio::test]
async fn create_task_encodes_project_segment_and_sends_scoped_json() {
    let response = r#"{"task":{"id":"task-1","title":"New task","project_id":"KAN / core","description":"line one\nline two","status":"to-do","priority":"high","number":42,"project_slug":"kan","assignee_name":null,"created_at":"2030-01-02T03:04:05Z","updated_at":null,"labels":[]}}"#;
    let (base_url, handle) = serve_once("200 OK", response);
    let request = CreateTaskRequest {
        title: "New task".to_owned(),
        body: "line one\nline two".to_owned(),
        status: Some("to-do".to_owned()),
        priority: Some("high".to_owned()),
    };

    let created = ApiClient::new(base_url)
        .create_board_task(TEAM_ID, PROVIDER_ID, "KAN / core", &request, "app-access")
        .await
        .expect("task should parse");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with(&format!(
        "POST /api/v1/task-board/providers/{PROVIDER_ID}/projects/KAN%20%2F%20core/tasks "
    )));
    assert!(wire.contains(&format!("x-team-id: {TEAM_ID}")));
    assert!(wire.contains("authorization: Bearer app-access"));
    assert!(wire.ends_with(
        r#"{"title":"New task","body":"line one\nline two","status":"to-do","priority":"high"}"#
    ));
    assert_eq!(created.task.id, "task-1");
}
