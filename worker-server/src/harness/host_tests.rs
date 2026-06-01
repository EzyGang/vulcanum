use crate::harness::host::HostHarness;
use crate::harness::{AgentHarness, ResourceLimits};

#[tokio::test]
async fn host_harness_timeout_or_error() {
    let harness = HostHarness::new();
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = std::collections::HashMap::new();
    let workdir = std::env::temp_dir();

    let result = harness
        .spawn("hello", &workdir, &secrets, &limits, "", "", "")
        .await;

    assert!(
        result.is_err(),
        "expected error (timeout or missing/invalid opencode)"
    );
}

#[tokio::test]
async fn host_harness_writes_agents_md() {
    let harness = HostHarness::new();
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-agents");
    let _ = std::fs::create_dir_all(&workdir);

    let agents_content = "# Test AGENTS.md\nThis is a test.";
    let _ = harness
        .spawn("test", &workdir, &secrets, &limits, "", agents_content, "")
        .await;

    let agents_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("AGENTS.md");
    let contents =
        std::fs::read_to_string(&agents_path).expect("AGENTS.md should have been written");
    let _ = std::fs::remove_dir_all(&workdir);

    assert_eq!(contents, agents_content);
}

#[tokio::test]
async fn host_harness_writes_opencode_config() {
    let harness = HostHarness::new();
    let limits = ResourceLimits {
        max_duration_secs: 1,
        ..Default::default()
    };
    let secrets = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-opencode-config");
    let _ = std::fs::create_dir_all(&workdir);

    let config_content = r#"{"providers":{"openai":{"apiKey":"{env:OPENAI_API_KEY}"}}}"#;
    let _ = harness
        .spawn("test", &workdir, &secrets, &limits, "", "", config_content)
        .await;

    let config_path = workdir
        .join("home")
        .join(".config")
        .join("opencode")
        .join("opencode.json");
    let contents =
        std::fs::read_to_string(&config_path).expect("opencode.json should have been written");
    let _ = std::fs::remove_dir_all(&workdir);

    assert_eq!(contents, config_content);
}
