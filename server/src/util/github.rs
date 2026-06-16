pub const GITHUB_REPO_URL_PREFIX: &str = "https://github.com/";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GithubRepo {
    owner: String,
    name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GithubPullRequest {
    repo: GithubRepo,
    number: i64,
    url: String,
}

impl GithubPullRequest {
    #[must_use]
    pub fn repo(&self) -> &GithubRepo {
        &self.repo
    }

    #[must_use]
    pub fn number(&self) -> i64 {
        self.number
    }

    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
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
pub fn parse_github_pr_url(value: &str) -> Option<GithubPullRequest> {
    let trimmed = value
        .split(['?', '#'])
        .next()
        .unwrap_or(value)
        .trim_end_matches('/');
    let path = github_repo_path(trimmed);
    let mut parts = path.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    let pull = parts.next()?;
    let number = parts.next()?;

    if parts.next().is_some() || pull != "pull" {
        return None;
    }

    let number = number.parse::<i64>().ok()?;
    let repo = GithubRepo {
        owner: owner.to_owned(),
        name: repo.to_owned(),
    };

    Some(GithubPullRequest {
        url: format!("{}{}", GITHUB_REPO_URL_PREFIX, path),
        repo,
        number,
    })
}

#[must_use]
fn github_repo_path(value: &str) -> &str {
    value
        .strip_prefix(GITHUB_REPO_URL_PREFIX)
        .or_else(|| value.strip_prefix("http://github.com/"))
        .or_else(|| value.strip_prefix("git@github.com:"))
        .or_else(|| value.strip_prefix("ssh://git@github.com/"))
        .unwrap_or(value)
}
