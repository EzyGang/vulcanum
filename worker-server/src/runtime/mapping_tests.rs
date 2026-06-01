use crate::runtime::client::events::SseEvent;
use crate::runtime::mapping;

fn make_sse(event_type: &str, properties: &str) -> SseEvent {
    SseEvent {
        event_type: event_type.to_owned(),
        properties: serde_json::from_str(properties).unwrap_or(serde_json::Value::Null),
    }
}

#[test]
fn session_status_busy_maps_to_turn_started() {
    let sse = make_sse(
        "session.status",
        r#"{"sessionID": "s1", "status": {"type": "busy"}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "turn.started");
}

#[test]
fn session_status_idle_maps_to_session_completed() {
    let sse = make_sse(
        "session.status",
        r#"{"sessionID": "s1", "status": {"type": "idle"}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.completed");
}

#[test]
fn session_status_retry_maps_to_turn_failed() {
    let sse = make_sse(
        "session.status",
        r#"{"sessionID": "s1", "status": {"type": "retry", "attempt": 1, "message": "err", "next": 0}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "turn.failed");
}

#[test]
fn session_idle_maps_to_session_completed() {
    let sse = make_sse("session.idle", r#"{"sessionID": "s1"}"#);
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.completed");
}

#[test]
fn session_error_maps_to_session_failed() {
    let sse = make_sse(
        "session.error",
        r#"{"sessionID": "s1", "error": {"name": "UnknownError", "data": {"message": "boom"}}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.failed");
}

#[test]
fn message_updated_assistant_maps_to_message_received() {
    let sse = make_sse(
        "message.updated",
        r#"{"info": {"role": "assistant", "tokens": {"input": 100, "output": 50}}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "message.received");
    assert_eq!(events[0].payload["role"], "assistant");
}

#[test]
fn message_updated_user_is_ignored() {
    let sse = make_sse("message.updated", r#"{"info": {"role": "user"}}"#);
    let events = mapping::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn message_part_updated_tool_running_maps_to_tool_called() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "tool", "tool": "bash", "state": {"status": "running"}}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "tool.called");
    assert_eq!(events[0].payload["tool"], "bash");
}

#[test]
fn message_part_updated_tool_completed_maps_to_tool_completed() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "tool", "tool": "edit", "state": {"status": "completed"}}}"#,
    );
    let events = mapping::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "tool.completed");
}

#[test]
fn message_part_updated_text_is_ignored() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "text", "text": "hello"}}"#,
    );
    let events = mapping::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn server_connected_is_ignored() {
    let sse = make_sse("server.connected", r#"{}"#);
    let events = mapping::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn unknown_event_type_is_ignored() {
    let sse = make_sse("custom.event", r#"{"foo": "bar"}"#);
    let events = mapping::map_event(&sse);
    assert!(events.is_empty());
}
