use std::path::Path;

use vulcanum_shared::api_types::{AgentBackend, JobRepo};
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
fn container_path_maps_host_path_under_workdir() {
    let workdir = Path::new("/tmp/vulcanum-work-job");
    let session_path = workdir
        .join("home")
        .join(".omp")
        .join("sessions")
        .join("session.jsonl");

    assert_eq!(
        workspace::container_path(workdir, "/workdir", &session_path),
        "/workdir/home/.omp/sessions/session.jsonl"
    );
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

#[tokio::test]
async fn surface_repo_context_copies_common_skill_roots() {
    let workspace_dir = std::env::temp_dir().join("vulcanum-test-common-skill-roots");
    let repo_dir = workspace_dir.join("repo");
    let _ = tokio::fs::remove_dir_all(&workspace_dir).await;
    tokio::fs::create_dir_all(&repo_dir)
        .await
        .expect("repo dir should be created");

    write_skill(&repo_dir, ".agents", "alpha", "agents-alpha").await;
    write_skill(&repo_dir, ".agents", "shared", "agents-shared").await;
    write_skill(&repo_dir, ".claude", "bravo", "claude-bravo").await;
    write_skill(&repo_dir, ".claude", "shared", "claude-shared").await;
    write_skill(&repo_dir, ".claude", "extra-shared", "claude-extra-shared").await;
    write_skill(&repo_dir, ".codex", "charlie", "codex-charlie").await;
    write_skill(&repo_dir, ".codex", "extra-shared", "codex-extra-shared").await;
    write_skill(&repo_dir, ".codex", "codex-omp-shared", "codex-omp-shared").await;
    write_skill(&repo_dir, ".omp", "delta", "omp-delta").await;
    write_skill(
        &repo_dir,
        ".omp",
        "codex-omp-shared",
        "omp-codex-omp-shared",
    )
    .await;
    write_skill(&repo_dir, ".omp", "shared", "omp-shared").await;
    tokio::fs::write(
        repo_dir.join(".claude").join("skills").join("ignored.md"),
        "ignored",
    )
    .await
    .expect("non-directory skill fixture should be written");

    workspace::surface_repo_context(&workspace_dir, &[repo_fixture()], AgentBackend::OpenCode)
        .await
        .expect("repo context should be surfaced");

    let skills_dir = workspace_dir.join(".agents").join("skills");
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("alpha").join("README.md"))
            .await
            .expect("alpha skill should be copied"),
        "agents-alpha"
    );
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("bravo").join("README.md"))
            .await
            .expect("bravo skill should be copied"),
        "claude-bravo"
    );
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("charlie").join("README.md"))
            .await
            .expect("charlie skill should be copied"),
        "codex-charlie"
    );
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("delta").join("README.md"))
            .await
            .expect("delta skill should be copied"),
        "omp-delta"
    );
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("shared").join("README.md"))
            .await
            .expect("first duplicate skill should be copied"),
        "agents-shared"
    );
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("extra-shared").join("README.md"))
            .await
            .expect("first extra duplicate skill should be copied"),
        "claude-extra-shared"
    );
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("codex-omp-shared").join("README.md"))
            .await
            .expect("codex duplicate skill should be copied before omp"),
        "codex-omp-shared"
    );
    assert!(!tokio::fs::try_exists(skills_dir.join("ignored.md"))
        .await
        .expect("ignored file existence should be checked"));

    assert!(
        !tokio::fs::try_exists(workspace_dir.join(".omp").join("skills"))
            .await
            .expect("omp skill target existence should be checked")
    );

    let _ = tokio::fs::remove_dir_all(&workspace_dir).await;
}

#[tokio::test]
async fn surface_repo_context_copies_skills_to_omp_target_for_omp_backend() {
    let workspace_dir = std::env::temp_dir().join("vulcanum-test-omp-skill-target");
    let repo_dir = workspace_dir.join("repo");
    let _ = tokio::fs::remove_dir_all(&workspace_dir).await;
    tokio::fs::create_dir_all(&repo_dir)
        .await
        .expect("repo dir should be created");

    write_skill(&repo_dir, ".agents", "alpha", "agents-alpha").await;

    workspace::surface_repo_context(&workspace_dir, &[repo_fixture()], AgentBackend::OmpRpc)
        .await
        .expect("repo context should be surfaced");

    let skills_dir = workspace_dir.join(".omp").join("skills");
    assert_eq!(
        tokio::fs::read_to_string(skills_dir.join("alpha").join("README.md"))
            .await
            .expect("alpha skill should be copied"),
        "agents-alpha"
    );
    assert!(
        !tokio::fs::try_exists(workspace_dir.join(".agents").join("skills"))
            .await
            .expect("opencode skill target existence should be checked")
    );

    let _ = tokio::fs::remove_dir_all(&workspace_dir).await;
}

#[tokio::test]
async fn surface_repo_context_ignores_absent_common_skill_roots() {
    let workspace_dir = std::env::temp_dir().join("vulcanum-test-absent-skill-roots");
    let repo_dir = workspace_dir.join("repo");
    let _ = tokio::fs::remove_dir_all(&workspace_dir).await;
    tokio::fs::create_dir_all(&repo_dir)
        .await
        .expect("repo dir should be created");

    workspace::surface_repo_context(&workspace_dir, &[repo_fixture()], AgentBackend::OpenCode)
        .await
        .expect("missing skill roots should be ignored");

    let skills_dir = workspace_dir.join(".agents").join("skills");
    let mut entries = tokio::fs::read_dir(&skills_dir)
        .await
        .expect("target skills dir should exist");
    assert!(entries
        .next_entry()
        .await
        .expect("target skills dir should be readable")
        .is_none());
    assert!(!tokio::fs::try_exists(workspace_dir.join("AGENTS.md"))
        .await
        .expect("workspace AGENTS.md existence should be checked"));

    let _ = tokio::fs::remove_dir_all(&workspace_dir).await;
}

async fn write_skill(repo_dir: &Path, root: &str, name: &str, contents: &str) {
    let skill_dir = repo_dir.join(root).join("skills").join(name);
    tokio::fs::create_dir_all(&skill_dir)
        .await
        .expect("skill fixture dir should be created");
    tokio::fs::write(skill_dir.join("README.md"), contents)
        .await
        .expect("skill fixture file should be written");
}

fn repo_fixture() -> WorkspaceRepo {
    WorkspaceRepo {
        full_name: "owner/repo".to_owned(),
        url: "https://github.com/owner/repo".to_owned(),
        relative_path: "repo".to_owned(),
    }
}
