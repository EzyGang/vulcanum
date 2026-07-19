use std::collections::HashMap;

use uuid::Uuid;
use vulcanum_shared::api::wire::AgentBackend;

use crate::isolation::github_credentials::GitHubCredentialBridge;
use crate::isolation::providers::docker::{
    build_container_environment, cleanup_docker_workdir, DockerIsolation,
};
use crate::isolation::providers::kata::KataIsolation;

#[test]
fn container_image_default() {
    let isolation = DockerIsolation::new(Some("test-runtime"), "test-image:v1".to_owned());
    assert!(!isolation.image.is_empty());
}

#[test]
fn docker_plain_no_runtime() {
    let isolation = DockerIsolation::new(None, "test-image:v1".to_owned());
    assert!(isolation.runtime.is_none());
    assert!(!isolation.image.is_empty());
}

#[test]
fn docker_environment_preserves_caller_env_vars() {
    let env_vars = HashMap::from([
        ("CALLER_FLAG".to_owned(), "enabled".to_owned()),
        ("CUSTOM_PATH".to_owned(), "/caller/bin".to_owned()),
    ]);
    let secrets = HashMap::from([
        ("API_KEY".to_owned(), "secret-value".to_owned()),
        ("GITHUB_TOKEN".to_owned(), "direct-token".to_owned()),
    ]);
    let github_credentials = GitHubCredentialBridge {
        host_env: HashMap::new(),
        runtime_env: HashMap::from([(
            "GIT_CONFIG_GLOBAL".to_owned(),
            "/workdir/home/.vulcanum/github/gitconfig".to_owned(),
        )]),
    };

    let (sanitized_secrets, combined_env) = build_container_environment(
        &env_vars,
        &secrets,
        &github_credentials,
        AgentBackend::OpenCode,
    );

    assert_eq!(
        combined_env.get("CALLER_FLAG").map(String::as_str),
        Some("enabled")
    );
    assert_eq!(
        combined_env.get("CUSTOM_PATH").map(String::as_str),
        Some("/caller/bin")
    );
    assert_eq!(
        combined_env.get("API_KEY").map(String::as_str),
        Some("secret-value")
    );
    assert!(!combined_env.contains_key("GITHUB_TOKEN"));
    assert_eq!(
        sanitized_secrets.get("API_KEY").map(String::as_str),
        Some("secret-value")
    );
    assert!(!sanitized_secrets.contains_key("GITHUB_TOKEN"));
}

#[test]
fn kata_inner_image_default() {
    let isolation = KataIsolation::new("test-image:v1".to_owned());
    assert!(!isolation.inner.image.is_empty());
}

#[test]
fn kata_inner_image_custom() {
    let isolation = KataIsolation::with_image("my-registry/agent:v1".to_owned());
    assert_eq!(isolation.inner.image, "my-registry/agent:v1");
}

#[tokio::test]
async fn docker_cleanup_removes_safe_workdir() {
    let workdir = std::env::temp_dir().join(format!("vulcanum-work-{}", Uuid::new_v4()));
    let nested = workdir.join("home").join("file.txt");
    tokio::fs::create_dir_all(nested.parent().expect("nested file should have parent"))
        .await
        .expect("workdir should be created");
    tokio::fs::write(&nested, "data")
        .await
        .expect("nested file should be written");

    cleanup_docker_workdir(&workdir, "unused-image", None).await;

    assert!(!workdir.exists());
}

#[tokio::test]
async fn docker_cleanup_refuses_unsafe_workdir() {
    let workdir = std::env::temp_dir().join(format!("vulcanum-other-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&workdir)
        .await
        .expect("workdir should be created");

    cleanup_docker_workdir(&workdir, "unused-image", None).await;

    assert!(workdir.exists());
    let _ = tokio::fs::remove_dir_all(&workdir).await;
}
