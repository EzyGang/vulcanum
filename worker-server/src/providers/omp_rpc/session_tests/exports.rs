use super::*;

#[tokio::test]
async fn agent_end_reads_documented_session_stats_tokens() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;

    session.map_frame(serde_json::json!({
        "type": "agent_end",
        "data": {
            "sessionFile": "C:\\Users\\Galtozzy\\.omp\\agent\\sessions\\session.jsonl",
            "sessionId": "019f2343-4e74-7000-abab-2b162f80b3dd",
            "userMessages": 0,
            "assistantMessages": 0,
            "toolCalls": 0,
            "toolResults": 0,
            "totalMessages": 0,
            "tokens": {
                "input": 11,
                "output": 7,
                "reasoning": 0,
                "cacheRead": 5,
                "cacheWrite": 3,
                "total": 26
            },
            "cost": 0,
            "premiumRequests": 0
        }
    }));

    assert_eq!(session.tokens.input, 11);
    assert_eq!(session.tokens.output, 7);
    assert_eq!(session.tokens.cache_read, 5);
    assert_eq!(session.tokens.cache_write, 3);
    assert_eq!(session.tokens.total, 26);
    Ok(())
}

#[tokio::test]
async fn configured_model_sets_export_model_used() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;
    let mut env = docker_env();
    env.env_vars
        .insert(VULCANUM_OMP_MODEL_ENV.to_owned(), "gpt-5-codex".to_owned());
    env.env_vars.insert(
        VULCANUM_OMP_PROVIDER_ENV.to_owned(),
        "openai-codex".to_owned(),
    );

    session.set_configured_model(&env);
    let export = session.export().await?;

    assert_eq!(
        export.model_used,
        Some("openai-codex/gpt-5-codex".to_owned())
    );
    Ok(())
}

#[tokio::test]
async fn configured_openai_model_sets_provider_prefixed_export_model_used(
) -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;
    let mut env = docker_env();
    env.env_vars
        .insert(VULCANUM_OMP_MODEL_ENV.to_owned(), "gpt-5.5".to_owned());
    env.env_vars
        .insert(VULCANUM_OMP_PROVIDER_ENV.to_owned(), "openai".to_owned());

    session.set_configured_model(&env);
    let export = session.export().await?;

    assert_eq!(export.model_used, Some("openai/gpt-5.5".to_owned()));
    Ok(())
}

#[tokio::test]
async fn reported_model_sets_provider_prefixed_model_used() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    let mut env = docker_env();
    env.env_vars.insert(
        VULCANUM_OMP_PROVIDER_ENV.to_owned(),
        "openai-codex".to_owned(),
    );
    env.env_vars.insert(
        VULCANUM_OMP_MODEL_ENV.to_owned(),
        "configured-fallback".to_owned(),
    );

    tx.send(serde_json::json!({
        "id": "state-1",
        "type": "response",
        "command": "get_state",
        "success": true,
        "data": {
            "sessionId": "abc",
            "model": "gpt-5-codex"
        }
    }))
    .await?;

    session.refresh_state(&env).await?;
    let export = session.export().await?;

    assert_eq!(
        export.model_used,
        Some("openai-codex/gpt-5-codex".to_owned())
    );
    Ok(())
}

#[test]
fn host_session_path_maps_container_session_file_to_host_path() {
    let env = docker_env();
    let expected_path = env
        .workdir
        .join("home")
        .join(".omp")
        .join("sessions")
        .join("session.jsonl")
        .to_string_lossy()
        .into_owned();

    assert_eq!(
        host_session_path(&env, "/workdir/home/.omp/sessions/session.jsonl"),
        expected_path
    );
}

#[tokio::test]
async fn export_messages_ignores_missing_session_file() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;
    session.session_path = std::env::temp_dir()
        .join("vulcanum-missing-session.jsonl")
        .to_string_lossy()
        .into_owned();

    let export = session.export_messages().await?;

    assert_eq!(export, None);
    Ok(())
}
