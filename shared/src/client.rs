mod auth;

use anyhow::Context;
use uuid::Uuid;

use crate::api_error::ApiError;
use crate::api_types::{
    AckRequest, AppendEventsRequest, AppendEventsResponse, ConnectRequest, ConnectResponse,
    JobResponse, PollResponse, RefreshGithubTokenResponse, RefreshRequest, RefreshResponse,
    StatusResponse, SubmitResultRequest, WireEvent, WorkerCapabilities,
};

#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn connect(
        &self,
        code: &str,
        worker_name: &str,
        max_concurrent_jobs: Option<i32>,
        capabilities: WorkerCapabilities,
    ) -> anyhow::Result<ConnectResponse> {
        let url = format!("{}/api/v1/workers/connect", self.base_url);
        let body = ConnectRequest {
            code: code.to_owned(),
            worker_name: worker_name.to_owned(),
            max_concurrent_jobs,
            capabilities,
        };
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
        let body = RefreshRequest {
            refresh_token: refresh_token.to_owned(),
        };
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("refresh request failed")?;

        map_response(resp).await.map_err(Into::into)
    }

    pub async fn delete_worker_self(&self, access_token: &str) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/workers/me", self.base_url);
        let resp = self
            .http
            .delete(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("self-delete request failed")?;

        if resp.status().is_success() {
            return Ok(());
        }

        Err(build_error(resp).await.into())
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

    pub async fn refresh_github_token(
        &self,
        job_id: Uuid,
        access_token: &str,
    ) -> anyhow::Result<RefreshGithubTokenResponse> {
        let url = format!("{}/api/v1/jobs/{}/github-token", self.base_url, job_id);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(access_token)
            .header(reqwest::header::CONTENT_LENGTH, "0")
            .body(String::new())
            .send()
            .await
            .context("refresh github token request failed")?;

        map_response(resp).await.map_err(Into::into)
    }

    pub async fn ack_job(&self, job_id: Uuid, access_token: &str) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/jobs/{}/ack", self.base_url, job_id);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(access_token)
            .json(&AckRequest {})
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

    pub async fn append_events(
        &self,
        job_id: Uuid,
        events: &[WireEvent],
        access_token: &str,
    ) -> anyhow::Result<AppendEventsResponse> {
        let url = format!("{}/api/v1/jobs/{}/events", self.base_url, job_id);
        let body = AppendEventsRequest {
            events: events.to_vec(),
        };
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .bearer_auth(access_token)
            .send()
            .await
            .context("append events request failed")?;

        map_response(resp).await.map_err(Into::into)
    }
}

pub async fn probe_url(url: &str) -> anyhow::Result<StatusResponse> {
    let client = ApiClient::new(url);
    client.status().await
}

pub async fn probe_url_with_scheme_fallback(
    user_url: &str,
) -> anyhow::Result<(String, StatusResponse)> {
    let user_url = user_url.trim();
    let normalized = if user_url.starts_with("http://") || user_url.starts_with("https://") {
        user_url.trim_end_matches('/').to_owned()
    } else {
        format!("https://{}", user_url.trim_end_matches('/'))
    };

    match probe_url(&normalized).await {
        Ok(resp) => Ok((normalized, resp)),
        Err(_) if normalized.starts_with("https://") => {
            let http_url = normalized.replacen("https://", "http://", 1);
            let resp = probe_url(&http_url).await?;
            Ok((http_url, resp))
        }
        Err(e) => Err(e),
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
