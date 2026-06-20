use crate::providers::opencode::event_mapper;
use crate::providers::opencode::events::SseEvent;

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
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "turn.started");
}

#[test]
fn session_status_idle_maps_to_session_completed() {
    let sse = make_sse(
        "session.status",
        r#"{"sessionID": "s1", "status": {"type": "idle"}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.completed");
}

#[test]
fn session_status_retry_maps_to_turn_failed() {
    let sse = make_sse(
        "session.status",
        r#"{"sessionID": "s1", "status": {"type": "retry", "attempt": 1, "message": "err", "next": 0}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "turn.failed");
}

#[test]
fn session_idle_maps_to_session_completed() {
    let sse = make_sse("session.idle", r#"{"sessionID": "s1"}"#);
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.completed");
}

#[test]
fn session_error_maps_to_session_failed() {
    let sse = make_sse(
        "session.error",
        r#"{"sessionID": "s1", "error": {"name": "UnknownError", "data": {"message": "boom"}}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.failed");
}

#[test]
fn message_updated_assistant_maps_to_message_received() {
    let sse = make_sse(
        "message.updated",
        r#"{"info": {"role": "assistant", "tokens": {"input": 100, "output": 50}}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "message.received");
    assert_eq!(events[0].payload["role"], "assistant");
}

#[test]
fn message_updated_assistant_finish_stop_does_not_complete_session() {
    let sse = make_sse(
        "message.updated",
        r#"{"info": {"role": "assistant", "sessionID": "s1", "finish": "stop", "tokens": {"input": 100, "output": 50}}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "message.received");
}

#[test]
fn message_updated_user_is_ignored() {
    let sse = make_sse("message.updated", r#"{"info": {"role": "user"}}"#);
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn session_next_step_ended_stop_maps_to_session_completed() {
    let sse = make_sse(
        "session.next.step.ended",
        r#"{"sessionID":"s1","assistantMessageID":"m1","finish":"stop","cost":0.1,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":3,"write":0}}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.completed");
    assert_eq!(events[0].payload["session_id"], "s1");
    assert_eq!(events[0].payload["reason"], "step_ended");
}

#[test]
fn session_next_step_ended_tool_calls_is_ignored() {
    let sse = make_sse(
        "session.next.step.ended",
        r#"{"sessionID":"s1","assistantMessageID":"m1","finish":"tool-calls","cost":0.1,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":3,"write":0}}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn session_next_step_failed_maps_to_session_failed() {
    let sse = make_sse(
        "session.next.step.failed",
        r#"{"sessionID":"s1","assistantMessageID":"m1","error":{"name":"UnknownError","data":{"message":"boom"}}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "session.failed");
}

#[test]
fn session_next_tool_success_maps_to_tool_completed() {
    let sse = make_sse(
        "session.next.tool.success",
        r#"{"sessionID":"s1","assistantMessageID":"m1","callID":"c1","tool":"bash","structured":{},"content":[],"provider":{"executed":true}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "tool.completed");
    assert_eq!(events[0].payload["tool"], "bash");
    assert_eq!(events[0].payload["status"], "completed");
}

#[test]
fn message_part_updated_tool_running_maps_to_tool_called() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "tool", "tool": "bash", "state": {"status": "running"}}}"#,
    );
    let events = event_mapper::map_event(&sse);
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
    let events = event_mapper::map_event(&sse);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "tool.completed");
}

#[test]
fn message_part_updated_step_finish_stop_is_ignored() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "step-finish", "sessionID": "s1", "reason": "stop"}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn message_part_updated_step_finish_tool_calls_is_ignored() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "step-finish", "sessionID": "s1", "reason": "tool-calls"}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn message_part_updated_text_is_ignored() {
    let sse = make_sse(
        "message.part.updated",
        r#"{"part": {"type": "text", "text": "hello"}}"#,
    );
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn server_connected_is_ignored() {
    let sse = make_sse("server.connected", r#"{}"#);
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}

#[test]
fn unknown_event_type_is_ignored() {
    let sse = make_sse("custom.event", r#"{"foo": "bar"}"#);
    let events = event_mapper::map_event(&sse);
    assert!(events.is_empty());
}
