use std::process::{Command, Stdio};
use std::time::Duration;

use tokio::process::Child;
use tokio::time::timeout;

const SERVER_CLEANUP_TIMEOUT_SECS: u64 = 10;

pub(crate) fn remove_container(name: Option<&str>) {
    let Some(name) = name else {
        return;
    };
    let _ = Command::new("docker")
        .args(["rm", "-f", name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub(crate) async fn stop_host_process(mut child: Child) {
    terminate_host_process_tree(child.id());
    let _ = child.start_kill();
    let _ = timeout(
        Duration::from_secs(SERVER_CLEANUP_TIMEOUT_SECS),
        child.wait(),
    )
    .await;
}

pub(crate) fn stop_host_process_sync(mut child: Child) {
    terminate_host_process_tree(child.id());
    let _ = child.start_kill();
}

fn terminate_host_process_tree(pid: Option<u32>) {
    let Some(pid) = pid else {
        return;
    };

    terminate_process_tree(pid);
}

#[cfg(windows)]
fn terminate_process_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(unix)]
fn terminate_process_tree(pid: u32) {
    let process_group = format!("-{pid}");
    let _ = Command::new("kill")
        .args(["-TERM", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(any(unix, windows)))]
fn terminate_process_tree(_pid: u32) {}
