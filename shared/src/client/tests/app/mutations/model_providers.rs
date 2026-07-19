use crate::api::app::model_providers::{
    ModelProviderAuthType, StartDeviceFlowRequest, UpdateModelProviderRequest,
};
use crate::client::tests::app::{assert_header, TEAM_ID};
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

#[tokio::test]
async fn update_provider_patches_only_explicit_fields() {
    let id = uuid::Uuid::from_u128(4);
    let response = format!(
        r#"{{"id":"{id}","display_name":"Claude","provider_key":"anthropic","auth_type":"none","credential_fields":[],"oauth":null}}"#
    );
    let (base_url, handle) = serve_once("200 OK", response);
    let request = UpdateModelProviderRequest {
        display_name: Some("Claude".to_owned()),
        auth_type: Some(ModelProviderAuthType::None),
        credentials: None,
    };
    ApiClient::new(base_url)
        .update_model_provider(TEAM_ID, id, &request, "app-access")
        .await
        .expect("provider update should parse");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with(&format!("PATCH /api/v1/model-providers/{id} ")));
    assert_header(&wire, "x-team-id", &TEAM_ID.to_string());
    assert!(wire.ends_with(r#"{"display_name":"Claude","auth_type":"none"}"#));
}

#[tokio::test]
async fn device_flow_start_posts_typed_request_and_response() {
    let response = r#"{"attempt_id":"00000000-0000-0000-0000-000000000005","verification_uri":"https://example.test/device","user_code":"ABCD","interval_seconds":5,"expires_at":"2030-01-02T03:04:05Z"}"#;
    let (base_url, handle) = serve_once("200 OK", response);
    let request = StartDeviceFlowRequest {
        provider_key: "openai".to_owned(),
        device_provider: "openai_chatgpt".to_owned(),
        display_name: String::new(),
    };
    let flow = ApiClient::new(base_url)
        .start_model_provider_device_flow(TEAM_ID, &request, "app-access")
        .await
        .expect("device flow should parse");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with("POST /api/v1/model-providers/device-flows "));
    assert_eq!(flow.user_code, "ABCD");
}
