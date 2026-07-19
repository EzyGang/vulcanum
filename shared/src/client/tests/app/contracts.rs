use crate::api::app::model_providers::PollDeviceFlowResponse;
use crate::api::app::teams::UpdateTeamModelsRequest;

#[test]
fn model_selection_distinguishes_omitted_and_null_fields() {
    let request = UpdateTeamModelsRequest {
        primary_model_provider_key: Some(None),
        primary_model_id: Some(None),
        small_model_provider_key: None,
        small_model_id: None,
    };
    let value = serde_json::to_value(request).expect("request should serialize");

    assert!(value["primary_model_provider_key"].is_null());
    assert!(value["primary_model_id"].is_null());
    assert!(value.get("small_model_provider_key").is_none());
    assert!(value.get("small_model_id").is_none());
}

#[test]
fn device_flow_decodes_pending_and_connected_variants() {
    let pending: PollDeviceFlowResponse =
        serde_json::from_str(r#"{"status":"pending","next_poll_at":"2030-01-02T03:04:05Z"}"#)
            .expect("pending response should decode");
    assert!(matches!(pending, PollDeviceFlowResponse::Pending { .. }));

    let connected: PollDeviceFlowResponse = serde_json::from_str(
        r#"{"status":"connected","provider":{"id":"00000000-0000-0000-0000-000000000004","display_name":"OpenAI","provider_key":"openai","auth_type":"device_oauth","credential_fields":[],"oauth":null}}"#,
    )
    .expect("connected response should decode");
    assert!(matches!(
        connected,
        PollDeviceFlowResponse::Connected { .. }
    ));
}
