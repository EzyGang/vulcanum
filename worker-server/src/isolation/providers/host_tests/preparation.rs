use super::*;

#[tokio::test]
async fn host_isolation_creates_workdir_and_config() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-host-isolation");

    let result = isolation
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            WorkRunType::Implementation,
            "# AGENTS.md",
            AgentBackend::OpenCode,
            &opencode_config(None),
            &[],
        )
        .await;

    match result {
        Ok(env) => {
            assert!(workdir.exists());
            let config_dir = workdir.join("home").join(".config").join("opencode");
            assert!(config_dir.join("AGENTS.md").exists());
            assert!(config_dir.join("opencode.json").exists());
            assert!(config_dir.join("tools").join("finish_run.ts").exists());
            assert_eq!(
                env.env_vars.get("OPENCODE_CONFIG"),
                Some(
                    &config_dir
                        .join("opencode.json")
                        .to_string_lossy()
                        .to_string()
                )
            );
            assert_eq!(
                env.env_vars.get("OPENCODE_CONFIG_DIR"),
                Some(&config_dir.to_string_lossy().to_string())
            );
            isolation.cleanup(&env).await;
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("git clone") || msg.contains("workdir"),
                "unexpected error: {msg}"
            );
        }
    }

    let _ = std::fs::remove_dir_all(&workdir);
}

#[tokio::test]
async fn host_isolation_writes_agents_md() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-host-agents");

    let agents_content = "# Test AGENTS.md\nThis is a test.";
    let result = isolation
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            WorkRunType::Implementation,
            agents_content,
            AgentBackend::OpenCode,
            &opencode_config(None),
            &[],
        )
        .await;

    let agents_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("AGENTS.md");
    let contents = std::fs::read_to_string(&agents_path);
    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(env) => {
            isolation.cleanup(&env).await;
            assert_eq!(contents.expect("AGENTS.md should exist"), agents_content);
        }
        Err(_) => {
            if let Ok(c) = contents {
                assert_eq!(c, agents_content);
            }
        }
    }
}

#[tokio::test]
async fn host_isolation_skips_agents_md_when_empty() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-host-no-agents");

    let _ = std::fs::create_dir_all(&workdir);
    let result = isolation
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            WorkRunType::Implementation,
            "",
            AgentBackend::OpenCode,
            &opencode_config(None),
            &[],
        )
        .await;

    let agents_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("AGENTS.md");
    let exists = agents_path.exists();
    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(env) => {
            isolation.cleanup(&env).await;
            assert!(!exists, "AGENTS.md should not be created when empty");
        }
        Err(_) => {
            assert!(!exists, "AGENTS.md should not be created when empty");
        }
    }
}

#[tokio::test]
async fn host_isolation_writes_generated_config() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-host-generated-config");

    let mut providers = std::collections::HashMap::new();
    providers.insert(
        "anthropic".to_owned(),
        OpenCodeProviderConfig {
            options: std::collections::HashMap::from([(
                "apiKey".to_owned(),
                "{env:ANTHROPIC_API_KEY}".to_owned(),
            )]),
        },
    );
    let _ = std::fs::create_dir_all(&workdir);
    let env = isolation
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            WorkRunType::Implementation,
            "",
            AgentBackend::OpenCode,
            &AgentConfigPayload::OpenCode {
                providers,
                model: Some("anthropic/claude-sonnet-4-5".to_owned()),
                small_model: Some("anthropic/claude-haiku-4-5".to_owned()),
                auth_content: None,
            },
            &[],
        )
        .await
        .expect("prepare should succeed");

    let config_dir = workdir.join("home").join(".config").join("opencode");
    let generated_contents = std::fs::read_to_string(config_dir.join("opencode.json"));
    isolation.cleanup(&env).await;

    let generated: serde_json::Value =
        serde_json::from_str(&generated_contents.expect("generated config should exist"))
            .expect("generated config should be valid json");
    assert_eq!(generated["model"], "anthropic/claude-sonnet-4-5");
    assert_eq!(generated["small_model"], "anthropic/claude-haiku-4-5");
    assert_eq!(
        generated["provider"]["anthropic"]["options"]["apiKey"],
        "{env:ANTHROPIC_API_KEY}"
    );
    assert_eq!(generated["permission"]["*"], "allow");
    assert_eq!(generated["permission"]["question"], "deny");
}
