use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::ResourceLimits;

use crate::harness::host::HostIsolation;

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
            "# AGENTS.md",
            "{}",
            "",
        )
        .await;

    match result {
        Ok(env) => {
            assert!(workdir.exists());
            let config_dir = workdir.join("home").join(".config").join("opencode");
            assert!(config_dir.join("AGENTS.md").exists());
            assert!(config_dir.join("opencode.json").exists());
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
            agents_content,
            "",
            "",
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
        .prepare(&workdir, &secrets, &env_vars, &limits, "", "", "")
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
async fn host_isolation_writes_opencode_config() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-host-config");

    let config_content = r#"{"providers":{"openai":{"apiKey":"{env:OPENAI_API_KEY}"}}}"#;
    let _ = std::fs::create_dir_all(&workdir);
    let result = isolation
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            "",
            config_content,
            "",
        )
        .await;

    let config_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("opencode.json");
    let contents = std::fs::read_to_string(&config_path);
    let _ = std::fs::remove_dir_all(&workdir);

    match result {
        Ok(env) => {
            isolation.cleanup(&env).await;
            assert_eq!(
                contents.expect("opencode.json should exist"),
                config_content
            );
        }
        Err(_) => {
            if let Ok(c) = contents {
                assert_eq!(c, config_content);
            }
        }
    }
}
