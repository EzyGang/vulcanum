use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GithubAppInstallation {
    pub id: i64,
    pub account_login: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct GithubAuthUrlResponse {
    pub url: String,
}
