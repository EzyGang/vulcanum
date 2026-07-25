use tokio::process::Command;

use crate::recovery::command;

use crate::state::journal::JournalEntry;

pub(super) async fn check_container_alive(entry: &JournalEntry) -> bool {
    let Some(name) = &entry.container_name else {
        return false;
    };

    let mut process = Command::new("docker");
    process.args(["inspect", "--format", "{{.State.Running}}", name]);
    match command::output(&mut process).await {
        Some(output) => String::from_utf8_lossy(&output.stdout).trim() == "true",
        None => false,
    }
}

pub(super) async fn check_host_alive(entry: &JournalEntry) -> bool {
    let pid = match entry.host_pid.or(entry.agent_pid) {
        Some(pid) => pid,
        None => return false,
    };

    check_process_alive(pid).await
}

#[cfg(unix)]
async fn check_process_alive(pid: i64) -> bool {
    let mut process = Command::new("kill");
    process.args(["-0", &pid.to_string()]);
    command::output(&mut process)
        .await
        .is_some_and(|output| output.status.success())
}

#[cfg(windows)]
async fn check_process_alive(pid: i64) -> bool {
    let filter = format!("PID eq {pid}");
    let mut process = Command::new("tasklist");
    process.args(["/FI", &filter, "/FO", "CSV", "/NH"]);
    match command::output(&mut process).await {
        Some(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .any(|line| line.contains(&format!(",\"{pid}\",")))
        }
        _ => false,
    }
}

#[cfg(not(any(unix, windows)))]
async fn check_process_alive(_pid: i64) -> bool {
    false
}
