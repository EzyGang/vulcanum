use std::collections::HashMap;
use std::path::PathBuf;

use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::providers::opencode::spawn::{container_docker_args, HOST_ENV_ALLOWLIST};

#[test]
fn host_env_allowlist_contains_expected_keys() {
    let expected = &["PATH", "TMPDIR", "HOME", "LANG"];
    for key in expected {
        assert!(
            HOST_ENV_ALLOWLIST.contains(key),
            "allowlist must contain {key}"
        );
    }
}

#[test]
fn host_env_allowlist_does_not_contain_sensitive_keys() {
    let sensitive = &[
        "GITHUB_TOKEN",
        "AWS_SECRET_ACCESS_KEY",
        "OPENAI_API_KEY",
        "KANEO_API_KEY",
    ];
    for key in sensitive {
        assert!(
            !HOST_ENV_ALLOWLIST.contains(key),
            "allowlist must not contain {key}"
        );
    }
}

#[test]
fn container_docker_args_passes_home_as_environment() {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    env_vars.insert("HOME".to_owned(), "/workdir/home".to_owned());
    env_vars.insert(
        "OPENCODE_CONFIG".to_owned(),
        "/workdir/home/.config/opencode/opencode.json".to_owned(),
    );
    env_vars.insert(
        "OPENCODE_CONFIG_DIR".to_owned(),
        "/workdir/home/.config/opencode".to_owned(),
    );
    env_vars.insert("OPENAI_API_KEY".to_owned(), "test-key".to_owned());

    let env = IsolatedEnvironment {
        workdir: PathBuf::from("/tmp/vulcanum-job"),
        workspace_dir: PathBuf::from("/tmp/vulcanum-job/workspace"),
        repos: Vec::new(),
        container_name: Some("vulcanum-job".to_owned()),
        secrets: HashMap::new(),
        env_vars,
        runtime: None,
        image: Some("agent-image:v1".to_owned()),
        server_host_port: None,
        limits: ResourceLimits::default(),
    };

    let args = container_docker_args(&env, "/workdir/workspace").unwrap();

    assert_env_arg(&args, "HOME=/workdir/home");
    assert_env_arg(
        &args,
        "OPENCODE_CONFIG=/workdir/home/.config/opencode/opencode.json",
    );
    assert_env_arg(&args, "OPENCODE_CONFIG_DIR=/workdir/home/.config/opencode");
    assert_env_arg(
        &args,
        "FINISH_ARTIFACT_PATH=/workdir/home/finish_artifact.json",
    );
    assert_env_arg(&args, "OPENAI_API_KEY=test-key");
    assert_eq!(
        args.iter()
            .filter(|arg| arg.as_str() == "HOME=/workdir/home")
            .count(),
        1
    );

    let image_index = arg_index(&args, "agent-image:v1");
    let home_index = arg_index(&args, "HOME=/workdir/home");
    assert!(home_index < image_index);
}

fn assert_env_arg(args: &[String], value: &str) {
    let index = arg_index(args, value);
    assert!(index > 0);
    assert_eq!(args[index - 1], "-e");
}

fn arg_index(args: &[String], value: &str) -> usize {
    match args.iter().position(|arg| arg == value) {
        Some(index) => index,
        None => panic!("missing docker arg {value}"),
    }
}
