pub const GITHUB_REPO_URL_PREFIX: &str = "https://github.com/";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GithubRepo {
    owner: String,
    name: String,
}

impl GithubRepo {
    #[must_use]
    pub fn owner(&self) -> &str {
        self.owner.as_str()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[must_use]
pub fn github_repo_url(full_name: &str) -> String {
    format!("{GITHUB_REPO_URL_PREFIX}{full_name}")
}

#[must_use]
pub fn github_repo_full_name_from_url(url: &str) -> String {
    match parse_github_repo(url) {
        Some(repo) => repo.full_name(),
        None => github_repo_path(url).trim_end_matches(".git").to_owned(),
    }
}

#[must_use]
pub fn parse_github_repo(value: &str) -> Option<GithubRepo> {
    let path = github_repo_path(value).trim_end_matches(".git");
    let (owner, name) = path.rsplit_once('/')?;

    if owner.is_empty() || name.is_empty() {
        return None;
    }

    Some(GithubRepo {
        owner: owner.to_owned(),
        name: name.to_owned(),
    })
}

#[must_use]
fn github_repo_path(value: &str) -> &str {
    value
        .strip_prefix(GITHUB_REPO_URL_PREFIX)
        .or_else(|| value.strip_prefix("http://github.com/"))
        .unwrap_or(value)
}
