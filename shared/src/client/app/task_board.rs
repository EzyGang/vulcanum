use uuid::Uuid;

use crate::api::app::task_board::{
    CreateTaskRequest, CreateTaskResponse, MoveTaskRequest, MoveTaskResponse, TaskBoardResponse,
    UpdateTaskRequest, UpdateTaskResponse,
};
use crate::client::ApiClient;

impl ApiClient {
    pub async fn get_task_board(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
        access_token: &str,
    ) -> anyhow::Result<TaskBoardResponse> {
        self.team_get(
            &format!(
                "task-board/providers/{provider_id}/projects/{}",
                encode_path_segment(external_project_id)
            ),
            team_id,
            access_token,
        )
        .await
    }

    pub async fn create_board_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
        request: &CreateTaskRequest,
        access_token: &str,
    ) -> anyhow::Result<CreateTaskResponse> {
        self.team_post(
            &format!(
                "task-board/providers/{provider_id}/projects/{}/tasks",
                encode_path_segment(external_project_id)
            ),
            team_id,
            request,
            access_token,
        )
        .await
    }

    pub async fn update_board_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        request: &UpdateTaskRequest,
        access_token: &str,
    ) -> anyhow::Result<UpdateTaskResponse> {
        self.team_patch(
            &format!(
                "task-board/providers/{provider_id}/tasks/{}",
                encode_path_segment(task_id)
            ),
            team_id,
            request,
            access_token,
        )
        .await
    }

    pub async fn move_board_task(
        &self,
        team_id: Uuid,
        provider_id: Uuid,
        task_id: &str,
        request: &MoveTaskRequest,
        access_token: &str,
    ) -> anyhow::Result<MoveTaskResponse> {
        self.team_patch(
            &format!(
                "task-board/providers/{provider_id}/tasks/{}/status",
                encode_path_segment(task_id)
            ),
            team_id,
            request,
            access_token,
        )
        .await
    }
}

fn encode_path_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}
