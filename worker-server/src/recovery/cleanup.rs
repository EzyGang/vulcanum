use tokio::process::Command;

use crate::recovery::command;

use crate::state::journal::JournalEntry;

pub(crate) async fn remove_container(name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
    let mut process = Command::new("docker");
    process.args(["rm", "-f", name]);
    let _ = command::output(&mut process).await;
}

pub(super) async fn cleanup_stale_job(entry: &JournalEntry) {
    if entry.harness_type == "host" {
        kill_host_process_group(entry).await;
    } else if let Some(name) = entry.container_name.as_deref() {
        remove_container(Some(name)).await;
    }
}

pub(crate) async fn kill_host_process_group(entry: &JournalEntry) {
    let pid = match entry.host_pid.or(entry.agent_pid) {
        Some(pid) => pid,
        None => return,
    };
    kill_process_tree(pid).await;
}

#[cfg(unix)]
async fn kill_process_tree(pid: i64) {
    let mut process = Command::new("kill");
    process.args(["-9", &format!("-{pid}")]);
    let _ = command::output(&mut process).await;
}

#[cfg(windows)]
async fn kill_process_tree(pid: i64) {
    let mut process = Command::new("taskkill");
    process.args(["/PID", &pid.to_string(), "/T", "/F"]);
    let _ = command::output(&mut process).await;
}

#[cfg(not(any(unix, windows)))]
async fn kill_process_tree(_pid: i64) {}
