use std::process::Stdio;

use crate::state::journal::JournalEntry;

pub(super) fn check_container_alive(entry: &JournalEntry) -> bool {
    let Some(name) = &entry.container_name else {
        return false;
    };

    let output = std::process::Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", name])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "true",
        Err(_) => false,
    }
}

pub(super) fn check_host_alive(entry: &JournalEntry) -> bool {
    let pid = match entry.host_pid.or(entry.agent_pid) {
        Some(pid) => pid,
        None => return false,
    };

    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
