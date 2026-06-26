use std::collections::HashMap;

use crate::isolation::github_credentials;

#[tokio::test]
async fn setup_writes_token_file_and_stable_helpers() {
    let workdir = std::env::temp_dir().join("vulcanum-test-github-bridge-setup");
    let _ = tokio::fs::remove_dir_all(&workdir).await;

    let bridge = github_credentials::setup(&workdir, Some("ghs_test"), "/workdir/home")
        .await
        .expect("credential bridge setup should succeed");

    let token_path = workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("token");
    let token = tokio::fs::read_to_string(&token_path)
        .await
        .expect("token file should exist");
    let gh_hosts = workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("gh-config")
        .join("hosts.yml");
    let git_config = workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("gitconfig");
    let gh_hosts_content = tokio::fs::read_to_string(&gh_hosts)
        .await
        .expect("gh hosts file should exist");
    let git_config_exists = git_config.exists();

    let _ = tokio::fs::remove_dir_all(&workdir).await;

    assert_eq!(token, "ghs_test");
    assert_eq!(
        bridge.runtime_env.get("VULCANUM_GITHUB_TOKEN_FILE"),
        Some(&"/workdir/home/.vulcanum/github/token".to_owned())
    );
    assert_eq!(
        bridge.runtime_env.get("GIT_CONFIG_GLOBAL"),
        Some(&"/workdir/home/.vulcanum/github/gitconfig".to_owned())
    );
    assert_eq!(
        bridge.runtime_env.get("GH_CONFIG_DIR"),
        Some(&"/workdir/home/.vulcanum/github/gh-config".to_owned())
    );
    assert!(gh_hosts_content.contains("oauth_token: 'ghs_test'"));
    assert!(gh_hosts_content.contains("git_protocol: https"));
    assert!(git_config_exists);
    assert!(!bridge.runtime_env.contains_key("GITHUB_TOKEN"));
    assert!(!bridge.runtime_env.contains_key("GH_TOKEN"));
    assert!(!bridge.runtime_env.contains_key("PATH"));
    assert!(!bridge
        .runtime_env
        .contains_key("VULCANUM_GITHUB_GH_WRAPPER"));
    assert!(!bridge.runtime_env.contains_key("VULCANUM_GITHUB_BIN_DIR"));
}

#[tokio::test]
async fn update_token_rewrites_and_removes_gh_hosts_file() {
    let workdir = std::env::temp_dir().join("vulcanum-test-github-bridge-refresh");
    let _ = tokio::fs::remove_dir_all(&workdir).await;

    github_credentials::setup(&workdir, Some("ghs_old"), "/workdir/home")
        .await
        .expect("credential bridge setup should succeed");
    github_credentials::update_token(&workdir, Some("ghs_new"))
        .await
        .expect("token refresh should succeed");

    let gh_hosts = workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("gh-config")
        .join("hosts.yml");
    let token_path = workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("token");
    let gh_hosts_content = tokio::fs::read_to_string(&gh_hosts)
        .await
        .expect("gh hosts file should exist");
    let token = tokio::fs::read_to_string(&token_path)
        .await
        .expect("token file should exist");

    assert_eq!(token, "ghs_new");
    assert!(gh_hosts_content.contains("oauth_token: 'ghs_new'"));
    assert!(!gh_hosts_content.contains("ghs_old"));

    github_credentials::update_token(&workdir, None)
        .await
        .expect("token removal should succeed");
    let gh_hosts_exists = gh_hosts.exists();
    let token_exists = token_path.exists();
    let _ = tokio::fs::remove_dir_all(&workdir).await;

    assert!(!gh_hosts_exists);
    assert!(!token_exists);
}

#[test]
fn token_env_filter_removes_direct_github_tokens() {
    let mut values = HashMap::new();
    values.insert("GITHUB_TOKEN".to_owned(), "one".to_owned());
    values.insert("GH_TOKEN".to_owned(), "two".to_owned());
    values.insert("OPENAI_API_KEY".to_owned(), "three".to_owned());

    let filtered = github_credentials::without_direct_token_env(&values);

    assert!(!filtered.contains_key("GITHUB_TOKEN"));
    assert!(!filtered.contains_key("GH_TOKEN"));
    assert_eq!(filtered.get("OPENAI_API_KEY"), Some(&"three".to_owned()));
}

#[test]
fn token_from_accepts_github_or_gh_token() {
    let mut values = HashMap::new();
    values.insert("GH_TOKEN".to_owned(), "ghs_two".to_owned());

    let token = github_credentials::token_from(&values);

    assert_eq!(token, Some("ghs_two"));
}
