use super::*;

#[tokio::test]
async fn wait_ready_reports_stderr_when_rpc_exits_before_ready() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    stderr.push_line("startup config missing".to_owned());
    let (mut session, tx) = test_session(stderr).await?;
    drop(tx);

    let error = session
        .wait_ready()
        .await
        .err()
        .ok_or("expected startup error")?;
    let message = error.to_string();

    assert!(message.contains("omp rpc exited before ready"));
    assert!(message.contains("stderr: startup config missing"));
    Ok(())
}

#[tokio::test]
async fn wait_ready_redacts_sensitive_stderr_when_rpc_exits() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    stderr.push_line("auth token missing".to_owned());
    let (mut session, tx) = test_session(stderr).await?;
    drop(tx);

    let error = session
        .wait_ready()
        .await
        .err()
        .ok_or("expected startup error")?;
    let message = error.to_string();

    assert!(message.contains("stderr: [redacted provider output]"));
    assert!(!message.contains("auth token missing"));
    Ok(())
}
