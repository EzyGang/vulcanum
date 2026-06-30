use std::process::Stdio;

use crate::state::journal::JournalEntry;

pub(crate) fn remove_container(name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
    let _ = std::process::Command::new("docker")
        .args(["rm", "-f", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub(super) fn cleanup_stale_job(entry: &JournalEntry) {
    if entry.harness_type == "host" {
        kill_host_process_group(entry);
    } else if let Some(name) = entry.container_name.as_deref() {
        remove_container(Some(name));
    }
}

pub(crate) fn kill_host_process_group(entry: &JournalEntry) {
    let pid = match entry.host_pid.or(entry.agent_pid) {
        Some(pid) => pid,
        None => return,
    };
    let _ = std::process::Command::new("kill")
        .args(["-9", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
