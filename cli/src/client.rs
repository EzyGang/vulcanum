use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct ConnectRequest<'a> {
    code: &'a str,
    worker_name: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ConnectResponse {
    pub access_token: String,
    pub refresh_token: String,
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
pub struct PollResponse {
    pub job_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct JobResponse {
    pub prompt_text: String,
    pub external_task_ref: String,
}

#[derive(Debug, Serialize)]
struct AckRequest {
    worker_id: Uuid,
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
        self.http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("connect request failed")?
            .error_for_status()
            .context("connect returned error status")?
            .json()
            .await
            .context("failed to parse connect response")
    }

    pub async fn refresh(&self, refresh_token: &str) -> anyhow::Result<RefreshResponse> {
        let url = format!("{}/api/v1/workers/refresh", self.base_url);
        let body = RefreshRequest { refresh_token };
        self.http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("refresh request failed")?
            .error_for_status()
            .context("refresh returned error status")?
            .json()
            .await
            .context("failed to parse refresh response")
    }

    pub async fn poll(&self, worker_id: Uuid, access_token: &str) -> anyhow::Result<Option<Uuid>> {
        let url = format!("{}/api/v1/poll", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("worker_id", worker_id.to_string())])
            .bearer_auth(access_token)
            .send()
            .await
            .context("poll request failed")?
            .error_for_status()
            .context("poll returned error status")?;

        if resp.status().as_u16() == 204 {
            return Ok(None);
        }

        let body: PollResponse = resp.json().await.context("failed to parse poll response")?;
        Ok(Some(body.job_id))
    }

    pub async fn get_job(&self, job_id: Uuid, access_token: &str) -> anyhow::Result<JobResponse> {
        let url = format!("{}/api/v1/jobs/{}", self.base_url, job_id);
        self.http
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("get job request failed")?
            .error_for_status()
            .context("get job returned error status")?
            .json()
            .await
            .context("failed to parse job response")
    }

    pub async fn ack_job(
        &self,
        job_id: Uuid,
        worker_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/jobs/{}/ack", self.base_url, job_id);
        let body = AckRequest { worker_id };
        self.http
            .post(&url)
            .json(&body)
            .bearer_auth(access_token)
            .send()
            .await
            .context("ack request failed")?
            .error_for_status()
            .context("ack returned error status")?;
        Ok(())
    }

    pub async fn submit_result(
        &self,
        job_id: Uuid,
        result: &SubmitResultRequest,
        access_token: &str,
    ) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/jobs/{}/result", self.base_url, job_id);
        self.http
            .post(&url)
            .json(result)
            .bearer_auth(access_token)
            .send()
            .await
            .context("submit result request failed")?
            .error_for_status()
            .context("submit result returned error status")?;
        Ok(())
    }
}
