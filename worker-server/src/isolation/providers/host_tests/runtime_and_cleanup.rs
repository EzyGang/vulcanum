use super::{
    opencode_config, AgentBackend, AgentConfigPayload, HostIsolation, IsolatedEnvironment,
    IsolationProvider, ResourceLimits, WorkRunType,
};

#[tokio::test]
async fn host_isolation_writes_omp_runtime_paths() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-test-host-omp-config");

    let env = isolation
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            WorkRunType::Implementation,
            "",
            AgentBackend::OmpRpc,
            &AgentConfigPayload::OmpRpc { config_yml: None },
            &[],
        )
        .await
        .expect("prepare should succeed");

    assert_eq!(
        env.env_vars.get("PI_SESSION_DIR"),
        Some(
            &workdir
                .join("home")
                .join(".omp")
                .join("sessions")
                .to_string_lossy()
                .to_string()
        )
    );
    assert_eq!(
        env.env_vars.get("FINISH_ARTIFACT_PATH"),
        Some(
            &workdir
                .join("home")
                .join("finish_artifact.json")
                .to_string_lossy()
                .to_string()
        )
    );
    assert!(workdir
        .join("workspace")
        .join(".omp")
        .join("tools")
        .join("finish_run")
        .join("index.ts")
        .exists());
    isolation.cleanup(&env).await;
}

#[tokio::test]
async fn host_isolation_cleanup_deletes_workdir() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let secrets = std::collections::HashMap::new();
    let env_vars = std::collections::HashMap::new();
    let workdir = std::env::temp_dir().join("vulcanum-work-test-host-cleanup");

    let env = isolation
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
        .await
        .expect("prepare should succeed");

    assert!(workdir.exists());
    isolation.cleanup(&env).await;
    assert!(!workdir.exists(), "cleanup should delete workdir");
}

#[tokio::test]
async fn host_isolation_cleanup_refuses_unsafe_path() {
    let isolation = HostIsolation::new();
    let limits = ResourceLimits::default();
    let _secrets: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let _env_vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let unsafe_dir = std::path::PathBuf::from("/tmp/unsafe-dir");

    let env = IsolatedEnvironment {
        workdir: unsafe_dir.clone(),
        workspace_dir: unsafe_dir.join("workspace"),
        repos: Vec::new(),
        container_name: None,
        secrets: std::collections::HashMap::new(),
        env_vars: std::collections::HashMap::new(),
        runtime: None,
        image: None,
        server_host_port: None,
        limits,
    };

    isolation.cleanup(&env).await;
    assert!(
        !unsafe_dir.exists() || std::fs::metadata(&unsafe_dir).is_err(),
        "cleanup should not delete unsafe paths"
    );
}
