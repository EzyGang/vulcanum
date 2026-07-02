use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api_types::{AgentBackend, AgentConfigPayload, JobResponse, WorkRunType};

use crate::recovery::recover_session::recovered_omp_env;
use crate::state::journal::{JournalEntry, JournalStatus};

#[tokio::test]
async fn recovered_omp_env_preserves_model_provider_env_for_docker() {
    let workdir = std::env::temp_dir().join(format!("vulcanum-omp-recovery-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&workdir)
        .await
        .expect("create workdir");

    let mut model_provider_env = HashMap::new();
    model_provider_env.insert("PI_PROVIDER".to_owned(), "openai-codex".to_owned());
    model_provider_env.insert("PI_MODEL".to_owned(), "gpt-5-codex".to_owned());
    model_provider_env.insert(
        "OPENAI_CODEX_OAUTH_TOKEN".to_owned(),
        "access-secret".to_owned(),
    );
    let job = test_job(model_provider_env, Some("github-secret".to_owned()));
    let entry = test_entry(&workdir, "docker");

    let env = recovered_omp_env(&entry, &job)
        .await
        .expect("recover OMP env");

    assert_eq!(
        env.env_vars.get("PI_PROVIDER"),
        Some(&"openai-codex".to_owned())
    );
    assert_eq!(
        env.env_vars.get("PI_MODEL"),
        Some(&"gpt-5-codex".to_owned())
    );
    assert_eq!(
        env.env_vars.get("OPENAI_CODEX_OAUTH_TOKEN"),
        Some(&"access-secret".to_owned())
    );
    assert!(!env.env_vars.contains_key("GITHUB_TOKEN"));
    assert!(env.env_vars.contains_key("VULCANUM_GITHUB_TOKEN_FILE"));
    assert_eq!(
        env.env_vars.get("PI_CONFIG_HOME"),
        Some(&"/workdir/home/.omp".to_owned())
    );
    assert_eq!(
        env.env_vars.get("PI_TMPDIR"),
        Some(&"/workdir/tmp".to_owned())
    );
    assert_eq!(env.container_name, Some("vulcanum-test".to_owned()));

    tokio::fs::remove_dir_all(&workdir)
        .await
        .expect("remove workdir");
}

fn test_job(
    model_provider_env: HashMap<String, String>,
    github_token: Option<String>,
) -> JobResponse {
    JobResponse {
        work_type: WorkRunType::Implementation,
        prompt_text: "do work".to_owned(),
        repos: Vec::new(),
        agents_md: String::new(),
        agent_backend: AgentBackend::OmpRpc,
        agent_config: AgentConfigPayload::OmpRpc { config_yml: None },
        model_provider_env,
        external_task_ref: "task".to_owned(),
        provider_instance_url: "https://kaneo.example".to_owned(),
        provider_api_key: "provider-secret".to_owned(),
        external_project_id: "project".to_owned(),
        external_workspace_id: "workspace".to_owned(),
        max_turns: 3,
        github_token,
        github_token_expires_at: None,
        pr_urls: Vec::new(),
        review_target_pr_url: None,
        review_target_repo_full_name: None,
    }
}

fn test_entry(workdir: &std::path::Path, harness_type: &str) -> JournalEntry {
    JournalEntry {
        job_id: Uuid::new_v4(),
        workdir: workdir.to_string_lossy().to_string(),
        container_name: Some("vulcanum-test".to_owned()),
        harness_type: harness_type.to_owned(),
        status: JournalStatus::Running,
        started_at: Utc::now(),
        finished_at: None,
        exit_code: None,
        tokens_used: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        pr_url: None,
        duration_ms: None,
        error_message: None,
        turn_count: Some(1),
        session_id: Some("session".to_owned()),
        max_turns: Some(3),
        host_pid: None,
        host_port: None,
        agent_backend: Some("omp".to_owned()),
        agent_session_path: Some("/workdir/home/.omp/sessions/session.jsonl".to_owned()),
        agent_config_dir: None,
        agent_state_dir: None,
        agent_transport: None,
        agent_pid: None,
    }
}
