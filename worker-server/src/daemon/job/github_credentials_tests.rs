use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use uuid::Uuid;
use vulcanum_shared::api::error::ApiError;
use vulcanum_shared::api::wire::GitCommitAuthor;

use crate::daemon::job::github_credentials::{commit_identity_env, is_retryable_refresh_error};

#[test]
fn client_errors_without_retry_semantics_stop_refresh_loop() {
    assert!(!is_retryable_refresh_error(&api_error(400)));
    assert!(!is_retryable_refresh_error(&api_error(403)));
    assert!(!is_retryable_refresh_error(&api_error(404)));
    assert!(!is_retryable_refresh_error(&api_error(409)));
}

#[test]
fn transient_http_errors_keep_refresh_loop_retrying() {
    assert!(is_retryable_refresh_error(&api_error(408)));
    assert!(is_retryable_refresh_error(&api_error(429)));
    assert!(is_retryable_refresh_error(&api_error(500)));
    assert!(is_retryable_refresh_error(&api_error(503)));
}

#[test]
fn non_http_errors_keep_refresh_loop_retrying() {
    let error = anyhow::anyhow!("network timeout");

    assert!(is_retryable_refresh_error(&error));
}

#[test]
fn generated_commit_uses_app_as_author_and_committer() {
    let author = GitCommitAuthor {
        name: "vulcanum-app[bot]".to_owned(),
        email: "123+vulcanum-app[bot]@users.noreply.github.com".to_owned(),
    };
    let env = commit_identity_env(Some(&author));
    let repo = std::env::temp_dir().join(format!("vulcanum-commit-identity-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&repo).expect("create test repository");
    run_git(&repo, &["init"], &env);
    std::fs::write(repo.join("change.txt"), "content").expect("write commit content");
    run_git(&repo, &["add", "change.txt"], &env);
    run_git(&repo, &["commit", "-m", "test identity"], &env);

    let output = run_git(
        &repo,
        &["show", "--no-patch", "--format=%an%n%ae%n%cn%n%ce"],
        &env,
    );
    let metadata = String::from_utf8(output.stdout)
        .expect("commit metadata should be UTF-8")
        .replace('\r', "");

    assert_eq!(
        metadata.lines().collect::<Vec<_>>(),
        vec![
            author.name.as_str(),
            author.email.as_str(),
            author.name.as_str(),
            author.email.as_str()
        ]
    );
    std::fs::remove_dir_all(repo).expect("remove test repository");
}

fn run_git(repo: &Path, args: &[&str], env: &HashMap<String, String>) -> std::process::Output {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .envs(env)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn api_error(status: u16) -> anyhow::Error {
    ApiError {
        status,
        body: String::new(),
    }
    .into()
}
