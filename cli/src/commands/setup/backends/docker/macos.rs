use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use vulcanum_shared::constants::MACOS_DOCKER_DESKTOP_CLI_PATH;

use crate::commands::setup::host::{macos_user, which_path};
use crate::console;

use super::{
    docker_binary_path, docker_desktop_launch_command, docker_info_status, DockerAccess, DOCKER_APP,
};

const DOCKER_START_ATTEMPTS: u16 = 120;
const DOCKER_START_DELAY: Duration = Duration::from_secs(2);

pub(super) fn install_docker() -> anyhow::Result<()> {
    let user_name = macos_user()?;
    if docker_binary_path().is_none() {
        install_docker_desktop(&user_name)?;
    } else if which_path("docker").is_none() {
        configure_installed_docker_desktop(&user_name)?;
    }

    launch_docker_desktop(&user_name)?;
    wait_for_docker_desktop()
}

#[must_use]
pub(super) fn docker_cli_path() -> Option<PathBuf> {
    let path = PathBuf::from(MACOS_DOCKER_DESKTOP_CLI_PATH);
    match path.is_file() {
        true => Some(path),
        false => None,
    }
}
pub(super) fn configure_docker_command(command: &mut Command) {
    let Some(resources_bin) = Path::new(MACOS_DOCKER_DESKTOP_CLI_PATH).parent() else {
        return;
    };
    let mut path = resources_bin.as_os_str().to_owned();
    match std::env::var_os("PATH") {
        Some(existing_path) if !existing_path.is_empty() => {
            path.push(":");
            path.push(existing_path);
        }
        Some(_) | None => (),
    }
    command.env("PATH", path);
}

pub(super) fn docker_desktop_install_command(app_path: &Path, user_name: &str) -> Command {
    let installer = app_path.join("Contents/MacOS/install");
    let mut command = Command::new("sudo");
    command
        .arg("-n")
        .arg(installer)
        .arg(format!("--user={user_name}"));
    command
}

fn install_docker_desktop(user_name: &str) -> anyhow::Result<()> {
    let arch = docker_desktop_arch()?;
    let url = format!("https://desktop.docker.com/mac/main/{arch}/Docker.dmg");
    let dmg_path = std::env::temp_dir().join(format!("vulcanum-docker-{arch}.dmg"));
    let mount_path = std::env::temp_dir().join(format!("vulcanum-docker-mount-{arch}"));

    console::progress(
        "Downloading Docker Desktop DMG",
        "Docker Desktop DMG download",
        || download_dmg(&url, &dmg_path),
    )?;
    console::progress(
        "Mounting Docker Desktop DMG",
        "Docker Desktop DMG mount",
        || attach_dmg(&dmg_path, &mount_path),
    )?;
    let install_result = console::progress(
        "Installing Docker.app from DMG",
        "Docker.app install",
        || run_docker_desktop_installer(&mount_path.join("Docker.app"), user_name),
    );
    detach_dmg(&mount_path);
    remove_downloaded_dmg(&dmg_path);
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

fn configure_installed_docker_desktop(user_name: &str) -> anyhow::Result<()> {
    console::progress(
        "Configuring Docker Desktop CLI tools",
        "Docker CLI tools",
        || run_docker_desktop_installer(Path::new(DOCKER_APP), user_name),
    )
}

fn run_docker_desktop_installer(app_path: &Path, user_name: &str) -> anyhow::Result<()> {
    let installer = app_path.join("Contents/MacOS/install");
    if !installer.is_file() {
        anyhow::bail!(
            "Docker Desktop installer not found at {}",
            installer.display()
        );
    }

    let status = docker_desktop_install_command(app_path, user_name)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to install Docker Desktop: {e}"))?;
    if !status.success() {
        anyhow::bail!("Docker Desktop installer failed");
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

fn remove_downloaded_dmg(dmg_path: &Path) {
    match std::fs::remove_file(dmg_path) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => {
            tracing::warn!(error = %err, path = %dmg_path.display(), "failed to remove Docker Desktop DMG")
        }
    }
}

fn launch_docker_desktop(user_name: &str) -> anyhow::Result<()> {
    let status = docker_desktop_launch_command(user_name)
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
