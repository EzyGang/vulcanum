use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use super::{docker_binary_path, docker_info_status, DockerAccess};

const DOCKER_APP: &str = "/Applications/Docker.app";
const DOCKER_CLI: &str = "/Applications/Docker.app/Contents/Resources/bin/docker";
const DOCKER_START_ATTEMPTS: u16 = 120;
const DOCKER_START_DELAY: Duration = Duration::from_secs(2);

pub(super) fn install_docker() -> anyhow::Result<()> {
    if docker_binary_path().is_none() {
        install_docker_desktop()?;
    }

    launch_docker_desktop()?;
    wait_for_docker_desktop()
}

#[must_use]
pub(super) fn docker_cli_path() -> Option<PathBuf> {
    let path = PathBuf::from(DOCKER_CLI);
    match path.is_file() {
        true => Some(path),
        false => None,
    }
}

fn install_docker_desktop() -> anyhow::Result<()> {
    let arch = docker_desktop_arch()?;
    let url = format!("https://desktop.docker.com/mac/main/{arch}/Docker.dmg");
    let dmg_path = std::env::temp_dir().join(format!("vulcanum-docker-{arch}.dmg"));
    let mount_path = std::env::temp_dir().join(format!("vulcanum-docker-mount-{arch}"));

    download_dmg(&url, &dmg_path)?;
    attach_dmg(&dmg_path, &mount_path)?;
    let install_result = install_app_from_mount(&mount_path);
    detach_dmg(&mount_path);
    install_result
}

fn docker_desktop_arch() -> anyhow::Result<&'static str> {
    match std::env::consts::ARCH {
        "aarch64" => Ok("arm64"),
        "x86_64" => Ok("amd64"),
        arch => anyhow::bail!("unsupported macOS CPU architecture for Docker Desktop: {arch}"),
    }
}

fn download_dmg(url: &str, dmg_path: &Path) -> anyhow::Result<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(dmg_path)
        .arg(url)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to download Docker Desktop: {e}"))?;
    if !status.success() {
        anyhow::bail!("failed to download Docker Desktop from {url}");
    }
    Ok(())
}

fn attach_dmg(dmg_path: &Path, mount_path: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(mount_path)
        .map_err(|e| anyhow::anyhow!("failed to create Docker Desktop mountpoint: {e}"))?;
    let status = Command::new("hdiutil")
        .arg("attach")
        .arg(dmg_path)
        .args([
            "-mountpoint",
            mount_path.to_string_lossy().as_ref(),
            "-nobrowse",
            "-quiet",
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to mount Docker Desktop DMG: {e}"))?;
    if !status.success() {
        anyhow::bail!("failed to mount Docker Desktop DMG");
    }
    Ok(())
}

fn install_app_from_mount(mount_path: &Path) -> anyhow::Result<()> {
    let source = mount_path.join("Docker.app");
    if !source.exists() {
        anyhow::bail!("Docker.app not found in mounted Docker Desktop DMG");
    }

    let status = Command::new("sudo")
        .arg("-n")
        .arg("ditto")
        .arg(&source)
        .arg(DOCKER_APP)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to install Docker.app: {e}"))?;
    if !status.success() {
        anyhow::bail!("failed to install Docker.app into /Applications");
    }
    Ok(())
}

fn detach_dmg(mount_path: &Path) {
    let status = Command::new("hdiutil")
        .arg("detach")
        .arg(mount_path)
        .arg("-quiet")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match status {
        Ok(_) => (),
        Err(err) => {
            tracing::warn!(error = %err, mount = %mount_path.display(), "failed to detach Docker Desktop DMG")
        }
    }
}

fn launch_docker_desktop() -> anyhow::Result<()> {
    let status = Command::new("open")
        .args(["-a", "Docker"])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to launch Docker Desktop: {e}"))?;
    if !status.success() {
        anyhow::bail!("failed to launch Docker Desktop");
    }
    Ok(())
}

fn wait_for_docker_desktop() -> anyhow::Result<()> {
    for attempt in 1..=DOCKER_START_ATTEMPTS {
        if docker_info_status(DockerAccess::Direct)? {
            return Ok(());
        }

        if attempt < DOCKER_START_ATTEMPTS {
            thread::sleep(DOCKER_START_DELAY);
        }
    }

    anyhow::bail!("Docker Desktop did not become ready; open Docker.app and complete any first-run prompts, then rerun setup")
}
