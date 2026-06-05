use crate::harness::prepare;

#[tokio::test]
async fn clone_repo_uses_isolated_git_config() {
    let dest = std::env::temp_dir().join("vulcanum-test-clone-isolated");
    let _ = tokio::fs::remove_dir_all(&dest).await;

    let result = prepare::clone_repo("https://github.com/octocat/Hello-World.git", &dest).await;

    let _ = tokio::fs::remove_dir_all(&dest).await;

    assert!(
        result.is_ok(),
        "clone with isolated git config should succeed for public repo"
    );
}
