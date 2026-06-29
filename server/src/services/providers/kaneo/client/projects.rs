use kaneo_cli::api::types::{Column, Project};

use crate::services::providers::kaneo::client::types::KaneoWorkspace;
use crate::services::providers::kaneo::client::{log_kaneo_result, KaneoClient};
use crate::services::providers::kaneo::errors::{api_err, KaneoError};

impl KaneoClient {
    pub async fn fetch_columns(&self, project_id: &str) -> Result<Vec<Column>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/column/{project_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }

    pub async fn lookup_project(&self, project_id: &str) -> Result<Project, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/project/{project_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }

    pub(crate) async fn fetch_workspaces(&self) -> Result<Vec<KaneoWorkspace>, KaneoError> {
        let client = self.build_client()?;
        let path = "/auth/organization/list";

        let start = std::time::Instant::now();
        let result = client.get(path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", path, duration_ms, &result);
        result
    }

    pub async fn fetch_projects(&self, workspace_id: &str) -> Result<Vec<Project>, KaneoError> {
        let client = self.build_client()?;
        let path = format!("/project?workspaceId={workspace_id}");

        let start = std::time::Instant::now();
        let result = client.get(&path).await.map_err(api_err);
        let duration_ms = start.elapsed().as_millis() as i64;

        log_kaneo_result("GET", &path, duration_ms, &result);
        result
    }
}
