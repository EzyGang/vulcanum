use vulcanum_shared::api_types::JobRepo;
use vulcanum_shared::runtime::types::WorkspaceRepo;

use crate::isolation::github_credentials;
use crate::isolation::workspace;

#[tokio::test]
async fn clone_repo_uses_isolated_git_config() {
    let workdir = std::env::temp_dir().join("vulcanum-test-clone-isolated");
    let dest = workdir.join("workspace").join("hello-world");
    let _ = tokio::fs::remove_dir_all(&workdir).await;
    tokio::fs::create_dir_all(dest.parent().expect("dest should have parent"))
        .await
        .expect("workspace dir should be created");
    let runtime_home = workdir.join("home").to_string_lossy().to_string();
    let credentials = github_credentials::setup(&workdir, None, &runtime_home)
        .await
        .expect("credential bridge should be created");

    let result = workspace::clone_repo(
        "https://github.com/octocat/Hello-World.git",
        &dest,
        &credentials.host_env,
    )
    .await;

    let _ = tokio::fs::remove_dir_all(&workdir).await;

    assert!(
        result.is_ok(),
        "clone with isolated git config should succeed for public repo"
    );
}

#[test]
fn redact_url_credentials_hides_authenticated_url_token() {
    let result = workspace::redact_url_credentials(
        "https://x-access-token:ghp_123@github.com/owner/repo.git",
    );

    assert_eq!(result, "https://<redacted>@github.com/owner/repo.git");
}

#[test]
fn workspace_repos_from_job_repos_keeps_clean_repo_url() {
    let repos = workspace::workspace_repos_from_job_repos(&[JobRepo {
        full_name: "owner/repo".to_owned(),
        url: "https://github.com/owner/repo.git".to_owned(),
    }]);

    assert_eq!(repos[0].url, "https://github.com/owner/repo.git");
    assert!(!repos[0].url.contains("ghp_"));
    assert!(!repos[0].url.contains("x-access-token"));
}

#[test]
fn repo_dir_name_uses_repo_basename() {
    let result = workspace::repo_dir_name("ezygang/actavoces");
    assert_eq!(result, "actavoces");
}

#[test]
fn repo_dir_name_sanitizes_basename() {
    let result = workspace::repo_dir_name("owner/My Repo.git");
    assert_eq!(result, "my-repo-git");
}

#[test]
fn workspace_prompt_prefix_requires_repo_commands_and_agents_chain() {
    let prompt = workspace::workspace_prompt_prefix(&[WorkspaceRepo {
        full_name: "owner/repo".to_owned(),
        url: "https://github.com/owner/repo".to_owned(),
        relative_path: "repo".to_owned(),
    }]);

    assert!(prompt.contains("wrapper workspace is not itself a repository"));
    assert!(prompt.contains("Run commands from the repository directory"));
    assert!(prompt.contains("aggregated ./AGENTS.md"));
    assert!(prompt.contains("down to the changed directories"));
    assert!(prompt.contains("formatter, validation, and test commands"));
}
