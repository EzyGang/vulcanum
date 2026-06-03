use serde::{Deserialize, Serialize};

use vulcanum_shared::runtime::errors::HarnessError;

use super::OpenCodeClient;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
pub enum OpenCodeSessionStatus {
    Idle,
    Busy,
    Retry {
        attempt: u32,
        message: String,
        next: u64,
    },
}

#[derive(Debug, Deserialize)]
pub struct Session {
    pub id: String,
    #[allow(dead_code)]
    pub title: String,
}

#[derive(Debug, Serialize)]
struct CreateSessionBody {
    title: String,
}

#[derive(Debug, Serialize)]
struct TextPart {
    #[serde(rename = "type")]
    part_type: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct SendMessageBody {
    parts: Vec<TextPart>,
}

pub async fn create_session(client: &OpenCodeClient, title: &str) -> Result<Session, HarnessError> {
    let url = format!("{}/session", client.base_url());
    let body = CreateSessionBody {
        title: title.to_owned(),
    };
    let resp = client
        .http_client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| HarnessError::Http(format!("create session failed: {e}")))?;

    let resp = OpenCodeClient::check_response(resp, |msg| {
        HarnessError::Http(format!("create session {msg}"))
    })
    .await?;

    resp.json::<Session>()
        .await
        .map_err(|e| HarnessError::Http(format!("create session parse failed: {e}")))
}

pub async fn send_message_async(
    client: &OpenCodeClient,
    session_id: &str,
    prompt: &str,
) -> Result<(), HarnessError> {
    let url = format!("{}/session/{session_id}/prompt_async", client.base_url());
    let body = SendMessageBody {
        parts: vec![TextPart {
            part_type: "text".to_owned(),
            text: prompt.to_owned(),
        }],
    };
    let resp = client
        .http_client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| HarnessError::Http(format!("send message failed: {e}")))?;

    OpenCodeClient::check_response(resp, |msg| {
        HarnessError::Http(format!("send message {msg}"))
    })
    .await?;

    Ok(())
}

pub async fn abort_session(client: &OpenCodeClient, session_id: &str) -> Result<(), HarnessError> {
    let url = format!("{}/session/{session_id}/abort", client.base_url());
    let resp = client
        .http_client()
        .post(&url)
        .send()
        .await
        .map_err(|e| HarnessError::CancelFailed(format!("abort request failed: {e}")))?;

    OpenCodeClient::check_response(resp, |msg| {
        HarnessError::CancelFailed(format!("abort {msg}"))
    })
    .await?;

    Ok(())
}

pub async fn get_session_status(
    client: &OpenCodeClient,
) -> Result<std::collections::HashMap<String, OpenCodeSessionStatus>, HarnessError> {
    let url = format!("{}/session/status", client.base_url());
    let resp = client
        .http_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| HarnessError::Http(format!("get session status failed: {e}")))?;

    let resp = OpenCodeClient::check_response(resp, |msg| {
        HarnessError::Http(format!("get session status {msg}"))
    })
    .await?;

    resp.json::<std::collections::HashMap<String, OpenCodeSessionStatus>>()
        .await
        .map_err(|e| HarnessError::Http(format!("parse session status failed: {e}")))
}
