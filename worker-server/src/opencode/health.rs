use serde::Deserialize;

use vulcanum_shared::runtime::errors::HarnessError;

use crate::opencode::OpenCodeClient;

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    #[allow(dead_code)]
    pub version: String,
}

pub async fn health_check(client: &OpenCodeClient) -> Result<HealthResponse, HarnessError> {
    let url = format!("{}/global/health", client.base_url());
    let resp = client
        .http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| HarnessError::Http(format!("health check request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(HarnessError::ServerUnhealthy(format!(
            "health check returned {}",
            resp.status()
        )));
    }

    resp.json::<HealthResponse>()
        .await
        .map_err(|e| HarnessError::Http(format!("health check parse failed: {e}")))
}
