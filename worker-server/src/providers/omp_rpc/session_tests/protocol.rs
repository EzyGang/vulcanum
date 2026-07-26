use super::{test_session, timeout, Duration, Error, ProcessOutputBuffer, Value};

#[tokio::test]
async fn wait_ready_preserves_startup_frames_until_ready() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    tx.send(serde_json::json!({"type": "extension_ui_request"}))
        .await?;
    tx.send(serde_json::json!({"type": "ready"})).await?;

    session.wait_ready().await?;

    assert_eq!(session.pending.len(), 1);
    assert_eq!(
        session
            .pending
            .front()
            .and_then(|frame| frame.get("type"))
            .and_then(Value::as_str),
        Some("extension_ui_request")
    );
    Ok(())
}

#[tokio::test]
async fn wait_for_response_keeps_pending_events_and_reads_new_frames() -> Result<(), Box<dyn Error>>
{
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    tx.send(serde_json::json!({"type": "message_update"}))
        .await?;
    tx.send(serde_json::json!({
        "type": "response",
        "id": "state-1",
        "command": "get_state",
        "success": true,
        "data": {"sessionId": "abc"}
    }))
    .await?;

    let response = timeout(
        Duration::from_secs(1),
        session.wait_for_response("state-1", "get_state"),
    )
    .await??;

    assert_eq!(
        response
            .get("data")
            .and_then(|data| data.get("sessionId"))
            .and_then(Value::as_str),
        Some("abc")
    );
    assert_eq!(session.pending.len(), 1);
    Ok(())
}

#[tokio::test]
async fn wait_for_response_times_out_when_command_never_answers() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;

    let result = session
        .wait_for_response_with_timeout("missing", "get_state", Duration::from_millis(1))
        .await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn wait_for_response_timeout_keeps_unmatched_frames_pending() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    tx.send(serde_json::json!({"type": "message_update", "id": "event-1"}))
        .await?;

    let result = session
        .wait_for_response_with_timeout("missing", "get_state", Duration::from_millis(1))
        .await;
    let error = result.err().ok_or("expected command timeout")?.to_string();

    assert!(error.contains("omp rpc command get_state response missing timed out"));
    assert_eq!(session.pending.len(), 1);
    assert_eq!(
        session
            .pending
            .front()
            .and_then(|frame| frame.get("type"))
            .and_then(Value::as_str),
        Some("message_update")
    );
    Ok(())
}
