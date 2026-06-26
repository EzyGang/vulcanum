mod scripts;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tokio::fs;
use vulcanum_shared::runtime::errors::HarnessError;

const GITHUB_TOKEN_KEYS: &[&str] = &["GITHUB_TOKEN", "GH_TOKEN"];

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GitHubCredentialBridge {
    pub(crate) host_env: HashMap<String, String>,
    pub(crate) runtime_env: HashMap<String, String>,
}

pub(crate) async fn setup(
    workdir: &Path,
    token: Option<&str>,
    runtime_home: &str,
) -> Result<GitHubCredentialBridge, HarnessError> {
    let home_dir = workdir.join("home");
    let github_dir = home_dir.join(".vulcanum").join("github");
    let gh_config_dir = github_dir.join("gh-config");

    fs::create_dir_all(&github_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create GitHub credential dir: {e}")))?;
    fs::create_dir_all(&gh_config_dir)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to create gh config dir: {e}")))?;

    update_token(workdir, token).await?;
    write_git_config(&github_dir.join("gitconfig")).await?;
    write_askpass(&github_dir.join("git-askpass.sh")).await?;
    write_askpass_cmd(&github_dir.join("git-askpass.cmd")).await?;

    Ok(GitHubCredentialBridge {
        host_env: host_command_env(workdir),
        runtime_env: runtime_env(runtime_home),
    })
}

pub(crate) async fn update_token(workdir: &Path, token: Option<&str>) -> Result<(), HarnessError> {
    let token_path = token_file(workdir);
    if let Some(parent) = token_path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            HarnessError::Crash(format!("failed to create GitHub credential dir: {e}"))
        })?;
    }

    match token {
        Some(token) => {
            validate_token(token)?;
            fs::write(&token_path, token).await.map_err(|e| {
                HarnessError::Crash(format!("failed to write GitHub token file: {e}"))
            })?;
            set_mode(&token_path, 0o600).await?;
            write_gh_hosts(workdir, token).await?;
        }
        None => {
            remove_file_if_exists(&token_path, "GitHub token file").await?;
            remove_file_if_exists(&gh_hosts_file(workdir), "gh hosts file").await?;
        }
    }

    Ok(())
}

#[must_use]
pub(crate) fn token_from(secrets: &HashMap<String, String>) -> Option<&str> {
    for key in GITHUB_TOKEN_KEYS {
        if let Some(token) = secrets.get(*key) {
            return Some(token.as_str());
        }
    }
    None
}

#[must_use]
pub(crate) fn without_direct_token_env(
    values: &HashMap<String, String>,
) -> HashMap<String, String> {
    values
        .iter()
        .filter(|(key, _)| !is_direct_token_key(key))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

#[must_use]
pub(crate) fn host_command_env(workdir: &Path) -> HashMap<String, String> {
    let home_dir = workdir.join("home");
    let github_dir = home_dir.join(".vulcanum").join("github");

    credential_env(
        path_value(github_dir.join("token")),
        path_value(github_dir.join("gitconfig")),
        path_value(github_dir.join(host_askpass_name())),
        path_value(github_dir.join("gh-config")),
    )
}

#[must_use]
pub(crate) fn runtime_env(runtime_home: &str) -> HashMap<String, String> {
    let github_dir = format!("{runtime_home}/.vulcanum/github");

    credential_env(
        format!("{github_dir}/token"),
        format!("{github_dir}/gitconfig"),
        format!("{github_dir}/git-askpass.sh"),
        format!("{github_dir}/gh-config"),
    )
}

fn credential_env(
    token_file: String,
    git_config: String,
    askpass: String,
    gh_config_dir: String,
) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("VULCANUM_GITHUB_TOKEN_FILE".to_owned(), token_file);
    env.insert("GIT_CONFIG_GLOBAL".to_owned(), git_config);
    env.insert("GIT_ASKPASS".to_owned(), askpass);
    env.insert("GH_CONFIG_DIR".to_owned(), gh_config_dir);
    env.insert("GIT_TERMINAL_PROMPT".to_owned(), "0".to_owned());
    env
}

fn token_file(workdir: &Path) -> PathBuf {
    workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("token")
}

fn gh_hosts_file(workdir: &Path) -> PathBuf {
    workdir
        .join("home")
        .join(".vulcanum")
        .join("github")
        .join("gh-config")
        .join("hosts.yml")
}

async fn write_git_config(path: &Path) -> Result<(), HarnessError> {
    fs::write(path, scripts::git_config())
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to write GitHub git config: {e}")))?;
    set_mode(path, 0o600).await
}

async fn write_askpass(path: &Path) -> Result<(), HarnessError> {
    fs::write(path, scripts::askpass_sh())
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to write GitHub askpass helper: {e}")))?;
    set_mode(path, 0o700).await
}

async fn write_askpass_cmd(path: &Path) -> Result<(), HarnessError> {
    fs::write(path, scripts::askpass_cmd())
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to write GitHub askpass helper: {e}")))?;
    set_mode(path, 0o700).await
}

async fn write_gh_hosts(workdir: &Path, token: &str) -> Result<(), HarnessError> {
    let hosts_path = gh_hosts_file(workdir);
    if let Some(parent) = hosts_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to create gh config dir: {e}")))?;
    }

    let content = format!(
        "github.com:\n  oauth_token: {}\n  git_protocol: https\n",
        yaml_single_quoted(token)
    );
    fs::write(&hosts_path, content)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to write gh hosts file: {e}")))?;
    set_mode(&hosts_path, 0o600).await
}

async fn remove_file_if_exists(path: &Path, description: &str) -> Result<(), HarnessError> {
    match fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(HarnessError::Crash(format!(
            "failed to remove {description}: {e}"
        ))),
    }
}

#[cfg(unix)]
async fn set_mode(path: &Path, mode: u32) -> Result<(), HarnessError> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions)
        .await
        .map_err(|e| HarnessError::Crash(format!("failed to set GitHub credential file mode: {e}")))
}

#[cfg(not(unix))]
async fn set_mode(_path: &Path, _mode: u32) -> Result<(), HarnessError> {
    Ok(())
}

fn is_direct_token_key(key: &str) -> bool {
    GITHUB_TOKEN_KEYS.contains(&key)
}

fn validate_token(token: &str) -> Result<(), HarnessError> {
    if token.contains(['\n', '\r']) {
        return Err(HarnessError::Crash(
            "GitHub token contains a line break".to_owned(),
        ));
    }

    Ok(())
}

fn yaml_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn path_value<P>(path: P) -> String
where
    P: AsRef<Path>,
{
    path.as_ref().to_string_lossy().to_string()
}

fn host_askpass_name() -> &'static str {
    match cfg!(windows) {
        true => "git-askpass.cmd",
        false => "git-askpass.sh",
    }
}
