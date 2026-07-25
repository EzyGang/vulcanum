use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;
use vulcanum_shared::api::wire::{AgentBackend, AgentConfigPayload, JobResponse, WorkRunType};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::worker::WorkerState;

use crate::recovery::recover_session::{mark_lost_and_submit, recovered_omp_env};
use crate::state::journal::{Journal, JournalEntry, JournalInsert, JournalStatus};

#[tokio::test]
async fn recovered_omp_env_preserves_model_provider_env_for_docker() {
    let workdir = std::env::temp_dir().join(format!("vulcanum-omp-recovery-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&workdir)
        .await
        .expect("create workdir");

    let mut model_provider_env = HashMap::new();
    model_provider_env.insert(
        "VULCANUM_OMP_PROVIDER".to_owned(),
        "openai-codex".to_owned(),
    );
    model_provider_env.insert("VULCANUM_OMP_MODEL".to_owned(), "gpt-5-codex".to_owned());
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
        env.env_vars.get("VULCANUM_OMP_PROVIDER"),
        Some(&"openai-codex".to_owned())
    );
    assert_eq!(
        env.env_vars.get("VULCANUM_OMP_MODEL"),
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

#[tokio::test]
async fn rejected_lost_result_removes_stale_journal_entry() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let base_url = format!("http://{}", listener.local_addr().expect("local address"));
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut buffer = [0_u8; 4096];
        let _bytes_read = stream.read(&mut buffer).expect("read request");
        let body = r#"{"error":"Invalid status transition"}"#;
        write!(
            stream,
            "HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("write response");
    });

    let path = std::env::temp_dir().join(format!("vulcanum-recovery-test-{}", Uuid::new_v4()));
    let journal = Arc::new(Journal::open(&path).expect("open journal"));
    let job_id = Uuid::new_v4();
    journal
        .insert_job(JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: Some("vulcanum-test"),
            harness_type: "docker",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("insert job");
    let entry = journal
        .find_by_id(job_id)
        .expect("read journal")
        .expect("journal entry");
    let client = Arc::new(ApiClient::new(base_url));
    let worker_state = Arc::new(RwLock::new(WorkerState {
        worker_id: Uuid::new_v4(),
        instance_url: String::new(),
        access_token: "token".to_owned(),
        refresh_token: "refresh".to_owned(),
        expires_at: Utc::now(),
        max_concurrent_jobs: 1,
    }));

    mark_lost_and_submit(&journal, &client, &worker_state, &entry).await;

    assert!(
        journal.find_by_id(job_id).expect("read journal").is_none(),
        "server-side reset must clear stale local state so redispatch can execute"
    );
    server.join().expect("join test server");
    drop(journal);
    std::fs::remove_file(path).expect("remove journal");
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
        github_commit_author: None,
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
        review_url: None,
        review_body: None,
        review_already_exists: false,
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
