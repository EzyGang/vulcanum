use std::sync::Arc;

use chrono::Utc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::harness::parse::{parse_pr_url, parse_token_usage};
use crate::state::journal::{Journal, JournalEntry, JournalStatus};

pub struct RecoveryResult {
    pub recovered_count: usize,
    pub monitors: Vec<JoinHandle<()>>,
}

pub async fn recover_running_jobs(
    journal: Arc<Journal>,
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
) -> RecoveryResult {
    let running = match journal.list_running() {
        Ok(jobs) => jobs,
        Err(e) => {
            tracing::warn!(error = %e, "failed to list running jobs, skipping recovery");
            return RecoveryResult {
                recovered_count: 0,
                monitors: Vec::new(),
            };
        }
    };

    if running.is_empty() {
        return RecoveryResult {
            recovered_count: 0,
            monitors: Vec::new(),
        };
    }

    tracing::info!(count = running.len(), "recovering running jobs from journal");

    let mut monitors = Vec::new();
    let mut recovered_count = 0usize;

    for entry in running {
        match entry.harness_type.as_str() {
            "host" => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    "host harness job cannot survive restart, marking lost"
                );
                let _ = journal.mark_lost(entry.job_id, "worker restarted, host process lost");
                submit_lost_result(client.clone(), &worker_state, &entry).await;
            }
            "kata" | "gvisor" => match check_container(&entry.container_name).await {
                ContainerStatus::Running => {
                    tracing::info!(
                        job_id = %entry.job_id,
                        container = %entry.container_name.as_deref().unwrap_or("?"),
                        "container still running, spawning monitor"
                    );
                    recovered_count += 1;
                    let handle = spawn_container_monitor(
                        client.clone(),
                        worker_state.clone(),
                        journal.clone(),
                        entry,
                    );
                    monitors.push(handle);
                }
                ContainerStatus::Exited => {
                    tracing::info!(
                        job_id = %entry.job_id,
                        "container exited, collecting logs and submitting result"
                    );
                    recover_exited_container(client.clone(), &worker_state, &journal, &entry).await;
                }
                ContainerStatus::NotFound => {
                    tracing::warn!(
                        job_id = %entry.job_id,
                        "container not found, marking lost"
                    );
                    let _ = journal.mark_lost(entry.job_id, "container not found after restart");
                    submit_lost_result(client.clone(), &worker_state, &entry).await;
                }
            },
            other => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    harness_type = %other,
                    "unknown harness type, marking lost"
                );
                let _ = journal.mark_lost(entry.job_id, "unknown harness type");
                submit_lost_result(client.clone(), &worker_state, &entry).await;
            }
        }
    }

    RecoveryResult {
        recovered_count,
        monitors,
    }
}

enum ContainerStatus {
    Running,
    Exited,
    NotFound,
}

async fn check_container(name: &Option<String>) -> ContainerStatus {
    let Some(name) = name else {
        return ContainerStatus::NotFound;
    };

    let output = match Command::new("docker")
        .args(["inspect", "--format", "{{.State.Status}}", name])
        .output()
        .await
    {
        Ok(o) => o,
        Err(_) => return ContainerStatus::NotFound,
    };

    if !output.status.success() {
        return ContainerStatus::NotFound;
    }

    let status = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    match status.as_str() {
        "running" => ContainerStatus::Running,
        "exited" => ContainerStatus::Exited,
        _ => ContainerStatus::NotFound,
    }
}

async fn get_container_exit_code(container_name: &str) -> Option<i32> {
    let output = Command::new("docker")
        .args([
            "inspect",
            "--format",
            "{{.State.ExitCode}}",
            container_name,
        ])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let code_str = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    code_str.parse::<i32>().ok()
}

async fn wait_container(container_name: &str) -> Option<i32> {
    let output = Command::new("docker")
        .args(["wait", container_name])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let code_str = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    code_str.parse::<i32>().ok()
}

async fn recover_exited_container(
    client: Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    journal: &Journal,
    entry: &JournalEntry,
) {
    let container_name = match &entry.container_name {
        Some(n) => n,
        None => {
            let _ = journal.mark_lost(entry.job_id, "no container name in journal entry");
            submit_lost_result(client, worker_state, entry).await;
            return;
        }
    };

    let exit_code = get_container_exit_code(container_name).await.unwrap_or(1);

    let logs = match collect_container_logs(container_name).await {
        Some(l) => l,
        None => {
            tracing::warn!(
                job_id = %entry.job_id,
                container = %container_name,
                "failed to collect container logs"
            );
            let _ = journal.mark_lost(entry.job_id, "failed to collect container logs");
            submit_lost_result(client, worker_state, entry).await;
            clean_up_container(container_name).await;
            return;
        }
    };

    let parsed_pr_url = parse_pr_url(&logs);
    let tokens_used = parse_token_usage(&logs);
    let duration_ms = Utc::now()
        .signed_duration_since(entry.started_at)
        .num_milliseconds()
        .max(0) as i64;

    let result = SubmitResultRequest {
        pr_url: parsed_pr_url.clone().unwrap_or_default(),
        exit_code,
        tokens_used: tokens_used as i64,
        duration_ms,
    };

    let journal_status = if exit_code == 0 {
        JournalStatus::Completed
    } else {
        JournalStatus::Failed
    };

    let _ = journal.update_result(
        entry.job_id,
        exit_code,
        tokens_used as i64,
        parsed_pr_url.as_deref(),
        duration_ms,
        journal_status,
    );

    let access_token = worker_state.read().await.access_token.clone();
    if let Err(e) = client
        .submit_result(entry.job_id, &result, &access_token)
        .await
    {
        tracing::warn!(job_id = %entry.job_id, error = %e, "failed to submit recovered result");
        return;
    }

    let _ = journal.mark_submitted(entry.job_id);
    clean_up_container(container_name).await;
}

fn spawn_container_monitor(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    entry: JournalEntry,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let container_name = match &entry.container_name {
            Some(n) => n.clone(),
            None => return,
        };

        tracing::info!(
            job_id = %entry.job_id,
            container = %container_name,
            "monitoring recovered container"
        );

        let exit_code = wait_container(&container_name).await.unwrap_or(1);

        let logs = match collect_container_logs(&container_name).await {
            Some(l) => l,
            None => {
                tracing::warn!(job_id = %entry.job_id, "failed to collect logs from recovered container");
                let _ = journal.mark_lost(
                    entry.job_id,
                    "failed to collect logs from recovered container",
                );
                submit_lost_result(client, &worker_state, &entry).await;
                clean_up_container(&container_name).await;
                return;
            }
        };

        let parsed_pr_url = parse_pr_url(&logs);
        let tokens_used = parse_token_usage(&logs);
        let duration_ms = Utc::now()
            .signed_duration_since(entry.started_at)
            .num_milliseconds()
            .max(0) as i64;

        let journal_status = if exit_code == 0 {
            JournalStatus::Completed
        } else {
            JournalStatus::Failed
        };

        let result = SubmitResultRequest {
            pr_url: parsed_pr_url.clone().unwrap_or_default(),
            exit_code,
            tokens_used: tokens_used as i64,
            duration_ms,
        };

        let _ = journal.update_result(
            entry.job_id,
            exit_code,
            tokens_used as i64,
            parsed_pr_url.as_deref(),
            duration_ms,
            journal_status,
        );

        let access_token = worker_state.read().await.access_token.clone();
        if let Err(e) = client
            .submit_result(entry.job_id, &result, &access_token)
            .await
        {
            tracing::warn!(job_id = %entry.job_id, error = %e, "failed to submit result from recovered container");
            return;
        }

        let _ = journal.mark_submitted(entry.job_id);
        clean_up_container(&container_name).await;
    })
}

async fn collect_container_logs(container_name: &str) -> Option<String> {
    let output = Command::new("docker")
        .args(["logs", container_name])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };

    Some(combined)
}

async fn clean_up_container(container_name: &str) {
    let mut child = match Command::new("docker")
        .args(["rm", "-f", container_name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                container = %container_name,
                error = %e,
                "failed to spawn docker rm"
            );
            return;
        }
    };

    let _ = child.wait().await;
}

async fn submit_lost_result(
    client: Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    entry: &JournalEntry,
) {
    let result = SubmitResultRequest {
        pr_url: String::new(),
        exit_code: 1,
        tokens_used: 0,
        duration_ms: Utc::now()
            .signed_duration_since(entry.started_at)
            .num_milliseconds()
            .max(0) as i64,
    };
    let access_token = worker_state.read().await.access_token.clone();
    if let Err(e) = client
        .submit_result(entry.job_id, &result, &access_token)
        .await
    {
        tracing::warn!(job_id = %entry.job_id, error = %e, "failed to submit lost result");
    }
}
