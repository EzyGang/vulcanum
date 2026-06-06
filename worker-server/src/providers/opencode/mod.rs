pub mod api;
pub mod cleanup;
pub mod event_mapper;
pub mod events;
pub mod health;
pub mod reporter;
pub mod runner;
pub mod runner_session;
pub mod runtime;
pub mod spawn;

use reqwest::Client;

use vulcanum_shared::runtime::errors::HarnessError;

#[derive(Clone)]
pub struct OpenCodeClient {
    http: Client,
    base_url: String,
}

impl OpenCodeClient {
    pub fn new(base_url: &str) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.to_owned(),
        }
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn http_client(&self) -> &Client {
        &self.http
    }

    pub(super) async fn check_response<F>(
        resp: reqwest::Response,
        err_fn: F,
    ) -> Result<reqwest::Response, HarnessError>
    where
        F: FnOnce(String) -> HarnessError,
    {
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(err_fn(format!("returned {status}: {text}")));
        }
        Ok(resp)
    }
}
