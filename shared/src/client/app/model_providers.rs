use uuid::Uuid;

use crate::api::app::model_providers::{
    AppModelProvider, CreateModelProviderRequest, ModelCatalog, PollDeviceFlowResponse,
    StartDeviceFlowRequest, StartDeviceFlowResponse, UpdateModelProviderRequest,
};
use crate::client::ApiClient;

impl ApiClient {
    pub async fn get_model_catalog(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<ModelCatalog> {
        self.team_get("model-providers/catalog", team_id, access_token)
            .await
    }

    pub async fn list_model_providers(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<AppModelProvider>> {
        self.team_get("model-providers", team_id, access_token)
            .await
    }

    pub async fn create_model_provider(
        &self,
        team_id: Uuid,
        request: &CreateModelProviderRequest,
        access_token: &str,
    ) -> anyhow::Result<AppModelProvider> {
        self.team_post("model-providers", team_id, request, access_token)
            .await
    }

    pub async fn update_model_provider(
        &self,
        team_id: Uuid,
        id: Uuid,
        request: &UpdateModelProviderRequest,
        access_token: &str,
    ) -> anyhow::Result<AppModelProvider> {
        self.team_patch(
            &format!("model-providers/{id}"),
            team_id,
            request,
            access_token,
        )
        .await
    }

    pub async fn delete_model_provider(
        &self,
        team_id: Uuid,
        id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<()> {
        self.team_delete(&format!("model-providers/{id}"), team_id, access_token)
            .await
    }

    pub async fn start_model_provider_device_flow(
        &self,
        team_id: Uuid,
        request: &StartDeviceFlowRequest,
        access_token: &str,
    ) -> anyhow::Result<StartDeviceFlowResponse> {
        self.team_post(
            "model-providers/device-flows",
            team_id,
            request,
            access_token,
        )
        .await
    }

    pub async fn poll_model_provider_device_flow(
        &self,
        team_id: Uuid,
        attempt_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<PollDeviceFlowResponse> {
        self.team_post(
            &format!("model-providers/device-flows/{attempt_id}/poll"),
            team_id,
            &(),
            access_token,
        )
        .await
    }
}
