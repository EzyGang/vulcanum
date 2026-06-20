use crate::providers::opencode::events::parse_sse_event_data;

#[test]
fn parse_sse_event_data_accepts_direct_event() {
    let event =
        parse_sse_event_data(r#"{"type":"session.idle","properties":{"sessionID":"ses_123"}}"#)
            .expect("direct event should parse");

    assert_eq!(event.event_type, "session.idle");
    assert_eq!(event.properties["sessionID"], "ses_123");
}

#[test]
fn parse_sse_event_data_accepts_global_event_wrapper() {
    let event = parse_sse_event_data(
        r#"{"directory":"/workdir/workspace","payload":{"id":"evt_1","type":"session.next.step.ended","properties":{"timestamp":1781978484612,"sessionID":"ses_123","assistantMessageID":"msg_1","finish":"stop","cost":0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":3,"write":0}}}}}"#,
    )
    .expect("wrapped global event should parse");

    assert_eq!(event.event_type, "session.next.step.ended");
    assert_eq!(event.properties["sessionID"], "ses_123");
    assert_eq!(event.properties["finish"], "stop");
}

#[test]
fn parse_sse_event_data_rejects_exported_message_history() {
    let event = parse_sse_event_data(
        r#"{"info":{"id":"msg_1","role":"assistant","sessionID":"ses_123"},"parts":[{"id":"prt_1","messageID":"msg_1","sessionID":"ses_123","type":"step-finish","reason":"stop"}]}"#,
    );

    assert!(event.is_none());
}
