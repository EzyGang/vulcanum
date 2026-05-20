use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api_error::ApiError;

#[derive(Debug, Serialize)]
struct ConnectRequest<'a> {
    code: &'a str,
    worker_name: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ConnectResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub worker_id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct RefreshRequest<'a> {
    refresh_token: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct StatusResponse {
    pub access_token_ttl_minutes: i64,
    pub code_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct PollResponse {
    pub job_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct JobResponse {
    pub prompt_text: String,
    pub external_task_ref: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitResultRequest {
    pub pr_url: String,
    pub exit_code: i32,
    pub tokens_used: i32,
    pub duration_ms: i32,
}

pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }

    pub async fn connect(&self, code: &str, worker_name: &str) -> anyhow::Result<ConnectResponse> {
        let url = format!("{}/api/v1/workers/connect", self.base_url);
        let body = ConnectRequest { code, worker_name };
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("connect request failed")?;

        map_response(resp).await.map_err(Into::into)
    }

    pub async fn refresh(&self, refresh_token: &str) -> anyhow::Result<RefreshResponse> {
        let url = format!("{}/api/v1/workers/refresh", self.base_url);
        let body = RefreshRequest { refresh_token };
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("refresh request failed")?;

        map_response(resp).await.map_err(Into::into)
    }

    pub async fn status(&self) -> anyhow::Result<StatusResponse> {
        let url = format!("{}/api/v1/status", self.base_url);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("status request failed")?;

        map_response(resp).await.map_err(Into::into)
    }

    pub async fn poll(&self, access_token: &str) -> anyhow::Result<Option<Uuid>> {
        let url = format!("{}/api/v1/poll", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("poll request failed")?;

        if resp.status().as_u16() == 204 {
            return Ok(None);
        }

        let body: PollResponse = map_response(resp).await?;
        Ok(Some(body.job_id))
    }

    pub async fn get_job(&self, job_id: Uuid, access_token: &str) -> anyhow::Result<JobResponse> {
        let url = format!("{}/api/v1/jobs/{}", self.base_url, job_id);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("get job request failed")?;

        map_response(resp).await.map_err(Into::into)
    }

    pub async fn ack_job(&self, job_id: Uuid, access_token: &str) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/jobs/{}/ack", self.base_url, job_id);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("ack request failed")?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(build_error(resp).await.into())
    }

    pub async fn submit_result(
        &self,
        job_id: Uuid,
        result: &SubmitResultRequest,
        access_token: &str,
    ) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/jobs/{}/result", self.base_url, job_id);
        let resp = self
            .http
            .post(&url)
            .json(result)
            .bearer_auth(access_token)
            .send()
            .await
            .context("submit result request failed")?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(build_error(resp).await.into())
    }
}

async fn build_error(resp: reqwest::Response) -> ApiError {
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    ApiError { status, body }
}

async fn map_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, ApiError> {
    if resp.status().is_success() {
        let status = resp.status().as_u16();
        return resp.json().await.map_err(|e| ApiError {
            status,
            body: e.to_string(),
        });
    }

    Err(build_error(resp).await)
}
