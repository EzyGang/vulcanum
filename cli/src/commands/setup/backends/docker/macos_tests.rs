use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use vulcanum_shared::constants::MACOS_DOCKER_DESKTOP_CLI_PATH;

use crate::commands::setup::backends::docker::docker_desktop_launch_command;
use crate::commands::setup::backends::docker::macos::{
    configure_docker_command, docker_desktop_install_command,
};

#[test]
fn docker_desktop_launches_in_login_user_context() {
    let command = docker_desktop_launch_command("worker-user");
    let arguments: Vec<&OsStr> = command.get_args().collect();

    assert_eq!(command.get_program(), OsStr::new("sudo"));
    assert_eq!(
        arguments,
        [
            OsStr::new("-u"),
            OsStr::new("worker-user"),
            OsStr::new("open"),
            OsStr::new("/Applications/Docker.app"),
        ]
    );
}

#[test]
fn docker_desktop_installer_configures_cli_tools_for_login_user() {
    let command =
        docker_desktop_install_command(Path::new("/Volumes/Docker/Docker.app"), "worker-user");
    let arguments: Vec<&OsStr> = command.get_args().collect();

    assert_eq!(command.get_program(), OsStr::new("sudo"));
    assert_eq!(
        arguments,
        [
            OsStr::new("-n"),
            OsStr::new("/Volumes/Docker/Docker.app/Contents/MacOS/install"),
            OsStr::new("--user=worker-user"),
        ]
    );
}

#[test]
fn docker_command_can_resolve_desktop_credential_helpers() {
    let mut command = Command::new("docker");
    configure_docker_command(&mut command);

    let configured_path = command
        .get_envs()
        .find(|(name, _)| *name == OsStr::new("PATH"))
        .and_then(|(_, value)| value);
    let resources_bin = Path::new(MACOS_DOCKER_DESKTOP_CLI_PATH).parent();
    let first_path = configured_path
        .and_then(|path| std::env::split_paths(path).next())
        .as_deref()
        .map(Path::to_path_buf);

    assert_eq!(first_path.as_deref(), resources_bin);
}
