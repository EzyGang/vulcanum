use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::providers::omp_rpc::process::{docker_omp_command, host_omp_command};

#[test]
fn host_omp_command_uses_yolo_flag() {
    let env = test_env(None);
    let command = host_omp_command(&env, None);
    let args = command_args(&command);

    assert!(args.contains(&OsString::from("--yolo")));
    assert!(!args.contains(&OsString::from("--auto-approve")));
}

#[test]
fn docker_omp_command_uses_yolo_flag() -> Result<(), Box<dyn std::error::Error>> {
    let env = test_env(Some("ghcr.io/ezygang/vulcanum/agent:latest".to_owned()));
    let command = docker_omp_command(&env, "vulcanum-test", None)?;
    let args = command_args(&command);

    assert!(args.contains(&OsString::from("--yolo")));
    assert!(!args.contains(&OsString::from("--auto-approve")));
    Ok(())
}

fn command_args(command: &tokio::process::Command) -> Vec<OsString> {
    command.as_std().get_args().map(OsString::from).collect()
}

fn test_env(image: Option<String>) -> IsolatedEnvironment {
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "PI_SESSION_DIR".to_owned(),
        "/workdir/home/.omp/sessions".to_owned(),
    );

    IsolatedEnvironment {
        workdir: PathBuf::from("/tmp/vulcanum-work-test"),
        workspace_dir: PathBuf::from("/tmp/vulcanum-work-test/workspace"),
        repos: Vec::new(),
        container_name: image.as_ref().map(|_| "vulcanum-test".to_owned()),
        secrets: HashMap::new(),
        env_vars,
        runtime: None,
        image,
        server_host_port: None,
        limits: ResourceLimits::default(),
    }
}
