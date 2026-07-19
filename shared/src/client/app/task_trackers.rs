use uuid::Uuid;

use crate::api::app::task_trackers::{
    CreateTaskTrackerRequest, TaskTracker, UpdateTaskTrackerRequest,
};
use crate::client::ApiClient;

impl ApiClient {
    pub async fn list_task_trackers(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<TaskTracker>> {
        self.team_get("providers", team_id, access_token).await
    }

    pub async fn create_task_tracker(
        &self,
        team_id: Uuid,
        request: &CreateTaskTrackerRequest,
        access_token: &str,
    ) -> anyhow::Result<TaskTracker> {
        self.team_post("providers", team_id, request, access_token)
            .await
    }

    pub async fn update_task_tracker(
        &self,
        team_id: Uuid,
        id: Uuid,
        request: &UpdateTaskTrackerRequest,
        access_token: &str,
    ) -> anyhow::Result<TaskTracker> {
        self.team_patch(&format!("providers/{id}"), team_id, request, access_token)
            .await
    }

    pub async fn delete_task_tracker(
        &self,
        team_id: Uuid,
        id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<()> {
        self.team_delete(&format!("providers/{id}"), team_id, access_token)
            .await
    }
}
