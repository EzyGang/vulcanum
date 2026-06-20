use eventsource_stream::Eventsource;
use futures::StreamExt;

use serde::Deserialize;

use super::OpenCodeClient;
use vulcanum_shared::runtime::errors::HarnessError;

#[derive(Debug, Clone, Deserialize)]
pub struct SseEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GlobalSseEvent {
    payload: SseEvent,
}

pub async fn connect_events(client: &OpenCodeClient) -> Result<SseEventStream, HarnessError> {
    let url = format!("{}/global/event", client.base_url());
    let resp = client
        .event_http_client()
        .get(&url)
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
                    match parse_sse_event_data(&data) {
                        Some(sse) => return Some(sse),
                        None => continue,
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

pub(crate) fn parse_sse_event_data(data: &str) -> Option<SseEvent> {
    match serde_json::from_str::<SseEvent>(data) {
        Ok(event) => Some(event),
        Err(_) => serde_json::from_str::<GlobalSseEvent>(data)
            .map(|event| event.payload)
            .ok(),
    }
}
