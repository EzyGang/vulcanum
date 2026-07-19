use crate::api::app::task_trackers::CreateTaskTrackerRequest;
use crate::client::tests::app::{assert_header, TEAM_ID};
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

const TRACKER_RESPONSE: &str = r#"{"id":"00000000-0000-0000-0000-000000000003","name":"Kaneo","provider_type":"kaneo","instance_url":"https://tasks.example"}"#;

#[tokio::test]
async fn create_tracker_posts_snake_case_body() {
    let (base_url, handle) = serve_once("200 OK", TRACKER_RESPONSE);
    let request = CreateTaskTrackerRequest {
        name: "Kaneo".to_owned(),
        instance_url: "https://tasks.example".to_owned(),
        api_key: "secret".to_owned(),
    };
    ApiClient::new(base_url)
        .create_task_tracker(TEAM_ID, &request, "app-access")
        .await
        .expect("tracker create should parse");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with("POST /api/v1/providers "));
    assert_header(&wire, "x-team-id", &TEAM_ID.to_string());
    assert!(wire.ends_with(
        r#"{"name":"Kaneo","instance_url":"https://tasks.example","api_key":"secret"}"#
    ));
}

#[tokio::test]
async fn delete_tracker_accepts_no_content() {
    let id = uuid::Uuid::from_u128(3);
    let (base_url, handle) = serve_once("204 No Content", "");
    ApiClient::new(base_url)
        .delete_task_tracker(TEAM_ID, id, "app-access")
        .await
        .expect("tracker delete should accept 204");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with(&format!("DELETE /api/v1/providers/{id} ")));
    assert_header(&wire, "authorization", "Bearer app-access");
}
