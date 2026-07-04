use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::providers::omp_rpc::process::{docker_omp_command, host_omp_command};
use crate::providers::omp_rpc::{
    VULCANUM_OMP_MODEL_ENV, VULCANUM_OMP_PROVIDER_ENV, VULCANUM_OMP_SMOL_ENV,
};

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

#[test]
fn host_omp_command_maps_launch_metadata() {
    let mut env = test_env(None);
    insert_omp_launch_metadata(&mut env);
    env.env_vars.insert(
        "OPENAI_CODEX_OAUTH_TOKEN".to_owned(),
        "access-secret".to_owned(),
    );

    let command = host_omp_command(&env, None);
    let args = command_args(&command);
    assert_arg_pair(&args, "--provider", "openai-codex");
    assert_arg_pair(&args, "--model", "gpt-5.5");
    assert_arg_pair(&args, "--smol", "anthropic/claude-haiku-4-5");
    assert_eq!(command_env_value(&command, VULCANUM_OMP_PROVIDER_ENV), None);
    assert_eq!(command_env_value(&command, VULCANUM_OMP_MODEL_ENV), None);
    assert_eq!(command_env_value(&command, VULCANUM_OMP_SMOL_ENV), None);
    assert_eq!(
        command_env_value(&command, "OPENAI_CODEX_OAUTH_TOKEN"),
        Some(OsString::from("access-secret"))
    );
}

#[test]
fn host_omp_command_removes_direct_github_token_env() {
    let mut env = test_env(None);
    env.env_vars
        .insert("GITHUB_TOKEN".to_owned(), "expired-token".to_owned());
    env.env_vars
        .insert("GH_TOKEN".to_owned(), "expired-gh-token".to_owned());

    let command = host_omp_command(&env, None);

    assert_eq!(command_env_value(&command, "GITHUB_TOKEN"), None);
    assert_eq!(command_env_value(&command, "GH_TOKEN"), None);
}

#[test]
fn docker_omp_command_maps_launch_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = test_env(Some("ghcr.io/ezygang/vulcanum/agent:latest".to_owned()));
    insert_omp_launch_metadata(&mut env);
    env.env_vars.insert(
        "OPENAI_CODEX_OAUTH_TOKEN".to_owned(),
        "access-secret".to_owned(),
    );

    let command = docker_omp_command(&env, "vulcanum-test", None)?;
    let args = command_args(&command);

    assert_arg_pair(&args, "--provider", "openai-codex");
    assert_arg_pair(&args, "--model", "gpt-5.5");
    assert_arg_pair(&args, "--smol", "anthropic/claude-haiku-4-5");
    assert!(!args.contains(&OsString::from(format!(
        "{VULCANUM_OMP_PROVIDER_ENV}=openai-codex"
    ))));
    assert!(!args.contains(&OsString::from(format!("{VULCANUM_OMP_MODEL_ENV}=gpt-5.5"))));
    assert!(!args.contains(&OsString::from(format!(
        "{VULCANUM_OMP_SMOL_ENV}=anthropic/claude-haiku-4-5"
    ))));
    assert!(args.contains(&OsString::from("OPENAI_CODEX_OAUTH_TOKEN=access-secret")));
    Ok(())
}

#[test]
fn docker_omp_command_does_not_pass_direct_github_token_env(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut env = test_env(Some("ghcr.io/ezygang/vulcanum/agent:latest".to_owned()));
    env.env_vars
        .insert("GITHUB_TOKEN".to_owned(), "expired-token".to_owned());
    env.env_vars
        .insert("GH_TOKEN".to_owned(), "expired-gh-token".to_owned());

    let command = docker_omp_command(&env, "vulcanum-test", None)?;
    let args = command_args(&command);

    assert!(!args.contains(&OsString::from("GITHUB_TOKEN=expired-token")));
    assert!(!args.contains(&OsString::from("GH_TOKEN=expired-gh-token")));
    Ok(())
}

fn command_args(command: &tokio::process::Command) -> Vec<OsString> {
    command.as_std().get_args().map(OsString::from).collect()
}

fn command_env_value(command: &tokio::process::Command, key: &str) -> Option<OsString> {
    command
        .as_std()
        .get_envs()
        .find(|(env_key, _)| *env_key == key)
        .and_then(|(_, value)| value.map(OsString::from))
}

fn assert_arg_pair(args: &[OsString], flag: &str, value: &str) {
    let expected_flag = OsString::from(flag);
    let expected_value = OsString::from(value);
    assert!(args
        .windows(2)
        .any(|pair| pair[0] == expected_flag && pair[1] == expected_value));
}

fn insert_omp_launch_metadata(env: &mut IsolatedEnvironment) {
    env.env_vars.insert(
        VULCANUM_OMP_PROVIDER_ENV.to_owned(),
        "openai-codex".to_owned(),
    );
    env.env_vars
        .insert(VULCANUM_OMP_MODEL_ENV.to_owned(), "gpt-5.5".to_owned());
    env.env_vars.insert(
        VULCANUM_OMP_SMOL_ENV.to_owned(),
        "anthropic/claude-haiku-4-5".to_owned(),
    );
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
