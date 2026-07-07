use std::collections::HashSet;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::{IntegrationBoard, IntegrationColumn};
use crate::models::task_board::errors::TaskBoardError;
use crate::models::task_board::model::TaskProviderProject;
use crate::services::providers::client::IntegrationClient;

const FALLBACK_STATUS: &str = "planned";

#[must_use]
pub(crate) fn project_config_to_provider_project(
    config: ProjectConfig,
) -> Option<TaskProviderProject> {
    let provider_id = config.provider_id?;
    let fallback_name = config.external_project_id.clone();

    Some(TaskProviderProject {
        provider_id,
        provider_type: config.integration_type,
        workspace_id: config.external_workspace_id,
        external_project_id: config.external_project_id,
        name: if config.name.is_empty() {
            fallback_name.clone()
        } else {
            config.name
        },
        slug: fallback_name,
    })
}

#[must_use]
pub(crate) fn collect_board_task_refs(board: &IntegrationBoard) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut task_refs = Vec::new();

    for column in &board.columns {
        for task in &column.tasks {
            let task_ref = task.id.clone();
            if seen.insert(task_ref.clone()) {
                task_refs.push(task_ref);
            }
        }
    }

    task_refs
}

#[must_use]
pub(crate) fn default_column_status(columns: &[IntegrationColumn]) -> String {
    columns
        .iter()
        .find(|column| column.is_final != Some(true))
        .or_else(|| columns.first())
        .map(|column| column.slug.to_owned())
        .unwrap_or_else(|| FALLBACK_STATUS.to_owned())
}

pub(crate) async fn default_task_status(
    client: &IntegrationClient,
    external_project_id: &str,
) -> Result<String, TaskBoardError> {
    let columns = client.fetch_columns(external_project_id).await?;
    Ok(default_column_status(&columns))
}

pub(crate) fn normalized_required(
    input: &str,
    err: TaskBoardError,
) -> Result<String, TaskBoardError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(err);
    }

    Ok(value.to_owned())
}
