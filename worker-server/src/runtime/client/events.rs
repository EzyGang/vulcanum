use eventsource_stream::Eventsource;
use futures::StreamExt;

use serde::Deserialize;

use crate::runtime::client::OpenCodeClient;
use vulcanum_shared::runtime::errors::HarnessError;

#[derive(Debug, Clone, Deserialize)]
pub struct SseEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

pub async fn connect_events(client: &OpenCodeClient) -> Result<SseEventStream, HarnessError> {
    let url = format!("{}/global/event", client.base_url());
    let resp = client
        .http_client()
        .get(&url)
        .basic_auth(client.username(), Some(client.password()))
        .send()
        .await
        .map_err(|e| HarnessError::Http(format!("events connect failed: {e}")))?;

    let resp = OpenCodeClient::check_response(resp, |msg| {
        HarnessError::Http(format!("events endpoint {msg}"))
    })
    .await?;

    Ok(SseEventStream::new(resp))
}

pub struct SseEventStream {
    inner: futures::stream::BoxStream<
        'static,
        Result<eventsource_stream::Event, eventsource_stream::EventStreamError<reqwest::Error>>,
    >,
    done: bool,
}

impl SseEventStream {
    fn new(response: reqwest::Response) -> Self {
        let stream = response.bytes_stream().eventsource().boxed();
        Self {
            inner: stream,
            done: false,
        }
    }

    pub async fn next_event(&mut self) -> Option<SseEvent> {
        if self.done {
            return None;
        }

        loop {
            let item = self.inner.next().await?;
            match item {
                Ok(event) => {
                    let data = event.data.trim().to_owned();
                    if data.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<SseEvent>(&data) {
                        Ok(sse) => return Some(sse),
                        Err(_) => continue,
                    }
                }
                Err(_) => {
                    self.done = true;
                    return None;
                }
            }
        }
    }
}
