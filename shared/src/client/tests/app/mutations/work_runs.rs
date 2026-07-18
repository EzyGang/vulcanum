use crate::api::app::work_runs::{WorkRunStatus, WorkRunType};
use crate::client::tests::app::{assert_header, TEAM_ID};
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

#[tokio::test]
async fn list_work_runs_gets_team_scoped_typed_items() {
    let response = r#"[{"id":"00000000-0000-0000-0000-000000000063","external_task_ref":"KAN-42","task_title":"Fix scheduler","status":"running","work_type":"implementation","tokens_used":1200,"input_tokens":700,"output_tokens":500,"cache_read_tokens":100,"cache_write_tokens":20,"model_used":"openai/gpt-5","duration_ms":4500,"created_at":"2030-01-02T03:04:05Z"}]"#;
    let (base_url, handle) = serve_once("200 OK", response);
    let runs = ApiClient::new(base_url)
        .list_work_runs(TEAM_ID, "app-access")
        .await
        .expect("work runs should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with("GET /api/v1/runs "));
    assert_header(&request, "authorization", "Bearer app-access");
    assert_header(&request, "x-team-id", &TEAM_ID.to_string());
    assert_eq!(runs[0].external_task_ref, "KAN-42");
    assert_eq!(runs[0].status, WorkRunStatus::Running);
    assert_eq!(runs[0].work_type, WorkRunType::Implementation);
    assert_eq!(runs[0].tokens_used, Some(1200));
}
