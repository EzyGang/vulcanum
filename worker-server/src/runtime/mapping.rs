use chrono::Utc;
use serde_json::json;

use vulcanum_shared::runtime::types::AgentEvent;

use crate::runtime::client::events::SseEvent;

pub fn map_event(sse: &SseEvent) -> Vec<AgentEvent> {
    let ts = Utc::now();
    match sse.event_type.as_str() {
        "session.status" => map_session_status(&sse.properties, ts),
        "session.idle" => map_session_idle(&sse.properties, ts),
        "session.error" => map_session_error(&sse.properties, ts),
        "message.updated" => map_message_updated(&sse.properties, ts),
        "message.part.updated" => map_message_part_updated(&sse.properties, ts),
        "server.connected" => vec![],
        _ => vec![],
    }
}

fn map_session_status(props: &serde_json::Value, ts: chrono::DateTime<Utc>) -> Vec<AgentEvent> {
    let status_type = props
        .get("status")
        .and_then(|s| s.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    match status_type {
        "busy" => vec![AgentEvent {
            event_type: "turn.started".to_owned(),
            payload: json!({"session_id": props.get("sessionID").and_then(|v| v.as_str()).unwrap_or("")}),
            timestamp: ts,
        }],
        "idle" => vec![AgentEvent {
            event_type: "session.completed".to_owned(),
            payload: json!({"session_id": props.get("sessionID").and_then(|v| v.as_str()).unwrap_or("")}),
            timestamp: ts,
        }],
        "retry" => vec![AgentEvent {
            event_type: "turn.failed".to_owned(),
            payload: props.clone(),
            timestamp: ts,
        }],
        _ => vec![],
    }
}

fn map_session_idle(props: &serde_json::Value, ts: chrono::DateTime<Utc>) -> Vec<AgentEvent> {
    vec![AgentEvent {
        event_type: "session.completed".to_owned(),
        payload: props.clone(),
        timestamp: ts,
    }]
}

fn map_session_error(props: &serde_json::Value, ts: chrono::DateTime<Utc>) -> Vec<AgentEvent> {
    vec![AgentEvent {
        event_type: "session.failed".to_owned(),
        payload: props.clone(),
        timestamp: ts,
    }]
}

fn map_message_updated(props: &serde_json::Value, ts: chrono::DateTime<Utc>) -> Vec<AgentEvent> {
    let role = props
        .get("info")
        .and_then(|i| i.get("role"))
        .and_then(|r| r.as_str())
        .unwrap_or("");

    match role {
        "assistant" => {
            let tokens = props
                .get("info")
                .and_then(|i| i.get("tokens"))
                .cloned()
                .unwrap_or(json!(null));

            vec![AgentEvent {
                event_type: "message.received".to_owned(),
                payload: json!({
                    "role": "assistant",
                    "tokens": tokens,
                }),
                timestamp: ts,
            }]
        }
        _ => vec![],
    }
}

fn map_message_part_updated(
    props: &serde_json::Value,
    ts: chrono::DateTime<Utc>,
) -> Vec<AgentEvent> {
    let part_type = props
        .get("part")
        .and_then(|p| p.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    if part_type != "tool" {
        return vec![];
    }

    let tool_name = props
        .get("part")
        .and_then(|p| p.get("tool"))
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    let tool_status = props
        .get("part")
        .and_then(|p| p.get("state"))
        .and_then(|s| s.get("status"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    vec![AgentEvent {
        event_type: match tool_status {
            "running" => "tool.called".to_owned(),
            "completed" | "error" => "tool.completed".to_owned(),
            "pending" => "tool.queued".to_owned(),
            _ => "tool.called".to_owned(),
        },
        payload: json!({
            "tool": tool_name,
            "status": tool_status,
        }),
        timestamp: ts,
    }]
}
