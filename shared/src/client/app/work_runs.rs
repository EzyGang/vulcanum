use uuid::Uuid;

use crate::api::app::work_runs::WorkRunListItem;
use crate::client::ApiClient;

impl ApiClient {
    pub async fn list_work_runs(
        &self,
        team_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<Vec<WorkRunListItem>> {
        self.team_get("runs", team_id, access_token).await
    }
}
