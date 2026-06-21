use vulcanum_shared::runtime::types::WorkspaceRepo;

use crate::isolation::workspace;

#[tokio::test]
async fn clone_repo_uses_isolated_git_config() {
    let dest = std::env::temp_dir().join("vulcanum-test-clone-isolated");
    let _ = tokio::fs::remove_dir_all(&dest).await;

    let result = workspace::clone_repo("https://github.com/octocat/Hello-World.git", &dest).await;

    let _ = tokio::fs::remove_dir_all(&dest).await;

    assert!(
        result.is_ok(),
        "clone with isolated git config should succeed for public repo"
    );
}

#[test]
fn authenticated_repo_url_injects_token_for_https_github() {
    let url = "https://github.com/owner/repo.git";
    let token = "ghp_123";
    let result = workspace::authenticated_repo_url(url, Some(token));
    assert_eq!(
        result,
        "https://x-access-token:ghp_123@github.com/owner/repo.git"
    );
}

#[test]
fn authenticated_repo_url_passes_through_without_token() {
    let url = "https://github.com/owner/repo.git";
    let result = workspace::authenticated_repo_url(url, None);
    assert_eq!(result, url);
}

#[test]
fn authenticated_repo_url_passes_through_non_https() {
    let url = "git@github.com:owner/repo.git";
    let result = workspace::authenticated_repo_url(url, Some("tok"));
    assert_eq!(result, url);
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
