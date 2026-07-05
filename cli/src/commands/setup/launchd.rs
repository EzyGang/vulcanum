use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::commands::setup::host::worker_server_path;

const LABEL: &str = "com.vulcanum.worker";
const PLIST_PATH: &str = "/Library/LaunchDaemons/com.vulcanum.worker.plist";
const SERVICE_PATH: &str = "system/com.vulcanum.worker";
const LAUNCHD_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/Applications/Docker.app/Contents/Resources/bin:/usr/bin:/bin:/usr/sbin:/sbin";

pub(crate) fn configure_worker_service() -> anyhow::Result<()> {
    let binary_path = worker_server_path()?;
    let user_name = service_user()?;
    let home_dir = service_home()?;
    let plist = launchd_plist(&binary_path, &user_name, &home_dir);
    let tmp_path = std::env::temp_dir().join("com.vulcanum.worker.plist");

    std::fs::write(&tmp_path, plist)
        .map_err(|e| anyhow::anyhow!("failed to write temporary launchd plist: {e}"))?;
    install_plist(&tmp_path)
}

#[must_use]
pub(crate) fn is_worker_service_installed() -> bool {
    match Path::new(PLIST_PATH).exists() {
        true => true,
        false => Command::new("launchctl")
            .args(["print", SERVICE_PATH])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false),
    }
}

pub(crate) fn enable_and_restart_worker_service() -> anyhow::Result<()> {
    bootout_best_effort();
    let status = Command::new("sudo")
        .args(["-n", "launchctl", "bootstrap", "system", PLIST_PATH])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to bootstrap launchd service: {e}"))?;
    if !status.success() {
        anyhow::bail!("launchctl bootstrap failed");
    }

    let status = Command::new("sudo")
        .args(["-n", "launchctl", "kickstart", "-k", SERVICE_PATH])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to kickstart launchd service: {e}"))?;
    if !status.success() {
        anyhow::bail!("launchctl kickstart failed");
    }

    Ok(())
}

pub(crate) fn remove_worker_service_best_effort() {
    bootout_best_effort();

    let status = Command::new("sudo")
        .args(["-n", "rm", "-f", PLIST_PATH])
        .status();
    match status {
        Ok(status) if status.success() => (),
        Ok(_) => tracing::warn!(path = PLIST_PATH, "failed to remove launchd plist"),
        Err(err) => {
            tracing::warn!(error = %err, path = PLIST_PATH, "failed to remove launchd plist")
        }
    }
}

fn install_plist(tmp_path: &Path) -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .args(["-n", "install", "-m", "0644"])
        .arg(tmp_path)
        .arg(PLIST_PATH)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to install launchd plist: {e}"))?;
    if !status.success() {
        anyhow::bail!("failed to install launchd plist");
    }
    Ok(())
}

fn bootout_best_effort() {
    let status = Command::new("sudo")
        .args(["-n", "launchctl", "bootout", SERVICE_PATH])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(_) => (),
        Err(err) => tracing::debug!(error = %err, "launchd bootout skipped"),
    }
}

fn launchd_plist(binary_path: &str, user_name: &str, home_dir: &Path) -> String {
    let binary_path = xml_escape(binary_path);
    let user_name = xml_escape(user_name);
    let home_dir = xml_escape(&home_dir.to_string_lossy());
    let path = xml_escape(LAUNCHD_PATH);

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
    <key>Label</key>\n\
    <string>{LABEL}</string>\n\
    <key>UserName</key>\n\
    <string>{user_name}</string>\n\
    <key>ProgramArguments</key>\n\
    <array>\n\
        <string>{binary_path}</string>\n\
    </array>\n\
    <key>WorkingDirectory</key>\n\
    <string>{home_dir}</string>\n\
    <key>EnvironmentVariables</key>\n\
    <dict>\n\
        <key>HOME</key>\n\
        <string>{home_dir}</string>\n\
        <key>PATH</key>\n\
        <string>{path}</string>\n\
    </dict>\n\
    <key>RunAtLoad</key>\n\
    <true/>\n\
    <key>KeepAlive</key>\n\
    <true/>\n\
    <key>StandardOutPath</key>\n\
    <string>/tmp/vulcanum-worker.log</string>\n\
    <key>StandardErrorPath</key>\n\
    <string>/tmp/vulcanum-worker.err</string>\n\
</dict>\n\
</plist>\n"
    )
}

fn service_user() -> anyhow::Result<String> {
    let user_name = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .map_err(|_| anyhow::anyhow!("failed to resolve current macOS user"))?;
    if user_name == "root" {
        anyhow::bail!("refusing to install launchd worker service as root; rerun setup from the target user account with passwordless sudo")
    }
    Ok(user_name)
}

fn service_home() -> anyhow::Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("failed to resolve user home directory"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
