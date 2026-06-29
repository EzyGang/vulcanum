mod labels;
mod projects;
mod tasks;
pub(crate) mod types;

use kaneo_cli::api::client::ApiClient;

use crate::services::providers::kaneo::errors::{api_err, KaneoError};

pub use crate::services::providers::kaneo::client::types::KaneoWorkspace;
#[cfg(test)]
pub(crate) use tasks::filter_tasks_in_column;

#[derive(Clone)]
pub struct KaneoClient {
    pub instance: String,
    pub api_key: String,
}

impl KaneoClient {
    pub fn new(instance: String, api_key: String) -> Self {
        Self { instance, api_key }
    }

    pub(crate) fn build_client(&self) -> Result<ApiClient, KaneoError> {
        ApiClient::new(&self.instance, &self.api_key).map_err(api_err)
    }
}

pub(crate) fn log_kaneo_result<T>(
    method: &str,
    path: &str,
    duration_ms: i64,
    result: &Result<T, KaneoError>,
) {
    match result {
        Ok(_) => {
            tracing::info!(
                method = method,
                path = path,
                duration_ms = duration_ms,
                "Kaneo API request succeeded",
            );
        }
        Err(error) => {
            tracing::warn!(
                method = method,
                path = path,
                duration_ms = duration_ms,
                error = %error,
                "Kaneo API request failed",
            );
        }
    }
}
