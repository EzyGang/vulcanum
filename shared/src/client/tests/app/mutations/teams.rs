use crate::api::app::teams::UpdateTeamModelsRequest;

use crate::client::tests::app::{assert_header, TEAM_ID, TEAM_RESPONSE};
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

#[tokio::test]
async fn update_models_patches_nested_nullable_fields() {
    let (base_url, handle) = serve_once("200 OK", TEAM_RESPONSE);
    let request = UpdateTeamModelsRequest {
        primary_model_provider_key: Some(Some("openai".to_owned())),
        primary_model_id: Some(Some("gpt-5".to_owned())),
        small_model_provider_key: None,
        small_model_id: None,
    };
    ApiClient::new(base_url)
        .update_team_models(TEAM_ID, &request, "app-access")
        .await
        .expect("team update should parse");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with(&format!("PATCH /api/v1/teams/{TEAM_ID} ")));
    assert_header(&wire, "authorization", "Bearer app-access");
    assert_header(&wire, "x-team-id", &TEAM_ID.to_string());
    assert!(wire.ends_with(r#"{"primary_model_provider_key":"openai","primary_model_id":"gpt-5"}"#));
}
