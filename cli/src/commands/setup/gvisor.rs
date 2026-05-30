use std::process::{Command, Stdio};

use super::utils::{docker_runtime_registered, read_daemon_json, run_systemctl, which};

pub(super) const GVISOR_RELEASE_BASE: &str =
    "https://storage.googleapis.com/gvisor/releases/release/latest";

pub fn install_gvisor() -> anyhow::Result<()> {
    if which("runsc") {
        tracing::debug!("runsc already installed");
        return Ok(());
    }

    let arch = std::env::consts::ARCH;
    let url = format!("{GVISOR_RELEASE_BASE}/{arch}");

    let script = format!(
        "set -e \n\
         TMPDIR=$(mktemp -d) \n\
         cd \"$TMPDIR\" \n\
         wget -q {url}/runsc {url}/runsc.sha512 \n\
         sha512sum -c runsc.sha512 \n\
         rm -f runsc.sha512 \n\
         chmod a+rx runsc \n\
         sudo mv runsc /usr/local/bin/runsc"
    );

    let status = Command::new("sh")
        .args(["-c", &script])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run gvisor install script: {e}"))?;

    if !status.success() && !which("runsc") {
        anyhow::bail!("gvisor installation failed");
    }

    Ok(())
}

pub fn configure_docker_for_gvisor() -> anyhow::Result<()> {
    if !which("runsc") {
        anyhow::bail!("runsc not found in PATH — install gvisor first");
    }

    if docker_runtime_registered("runsc") {
        tracing::info!("gVisor runtime already active in Docker — skipping configuration");
        return Ok(());
    }

    match read_daemon_json() {
        Ok(config) => {
            let in_config = config
                .get("runtimes")
                .and_then(|r| r.get("runsc"))
                .is_some();

            if in_config {
                tracing::debug!(
                    "runsc in daemon.json but not picked up by Docker, restarting daemon"
                );
                run_systemctl("restart docker")?;
                return Ok(());
            }
        }
        Err(_) => {
            tracing::debug!("no existing daemon.json found, will create one");
        }
    }

    let status = Command::new("sudo")
        .args(["/usr/local/bin/runsc", "install"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run runsc install: {e}"))?;

    if !status.success() {
        anyhow::bail!("runsc install failed");
    }

    run_systemctl("restart docker")?;
    Ok(())
}
