use std::ffi::OsStr;

use crate::commands::setup::backends::docker::docker_desktop_launch_command;

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
