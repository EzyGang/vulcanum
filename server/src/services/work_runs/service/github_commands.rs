use uuid::Uuid;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::WorkRunsService;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct GithubCommandAuthorizationRequest<'a> {
    pub installation_id: i64,
    pub sender_id: &'a str,
    pub single_user_mode: bool,
    pub repo_full_name: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GithubProjectOption {
    pub project_config_id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct GithubCommandResponseOptions {
    pub team_id: Uuid,
    pub projects: Vec<GithubProjectOption>,
}

pub(crate) enum GithubCommandAuthorization {
    Authorized {
        team_id: Uuid,
        projects: Vec<ProjectConfig>,
    },
    Unauthorized {
        team_id: Uuid,
    },
    UnknownInstallation,
    NoMatchingProject {
        team_id: Uuid,
    },
}

pub(crate) enum ProjectSelection<'a> {
    Selected(&'a ProjectConfig),
    Disabled,
    Required,
    Invalid,
}

impl WorkRunsService {
    pub(crate) async fn authorize_github_command(
        &self,
        request: GithubCommandAuthorizationRequest<'_>,
    ) -> Result<GithubCommandAuthorization, WorkRunsError> {
        let team_id = match self
            .github
            .repo
            .find_team_id_by_github_installation(&self.db, request.installation_id)
            .await?
        {
            Some(team_id) => team_id,
            None => return Ok(GithubCommandAuthorization::UnknownInstallation),
        };
        let linked_identity = match request.single_user_mode {
            true => {
                self.github
                    .repo
                    .is_linked_review_identity(&self.db, request.installation_id, request.sender_id)
                    .await?
            }
            false => false,
        };
        let authorized = match linked_identity {
            true => true,
            false => {
                self.project_configs
                    .teams
                    .is_provider_identity_member(team_id, "github", request.sender_id)
                    .await?
            }
        };
        if !authorized {
            return Ok(GithubCommandAuthorization::Unauthorized { team_id });
        }

        let projects = self
            .project_configs
            .list_enabled_for_github_repo(request.installation_id, request.repo_full_name)
            .await?;
        if projects.is_empty() {
            return Ok(GithubCommandAuthorization::NoMatchingProject { team_id });
        }

        Ok(GithubCommandAuthorization::Authorized { team_id, projects })
    }
}

pub(crate) fn select_project<'a>(
    selector: Option<&str>,
    enabled: &'a [ProjectConfig],
    disabled: &[ProjectConfig],
) -> ProjectSelection<'a> {
    match selector {
        Some(selector) => {
            let id = match selector
                .get(..8)
                .filter(|prefix| prefix.eq_ignore_ascii_case("project:"))
                .and_then(|_| Uuid::parse_str(&selector[8..]).ok())
            {
                Some(id) => id,
                None => return ProjectSelection::Invalid,
            };
            match enabled.iter().find(|config| config.id == id) {
                Some(config) => ProjectSelection::Selected(config),
                None if disabled.iter().any(|config| config.id == id) => ProjectSelection::Disabled,
                None => ProjectSelection::Invalid,
            }
        }
        None => match enabled {
            [config] => ProjectSelection::Selected(config),
            [] => ProjectSelection::Disabled,
            _ => ProjectSelection::Required,
        },
    }
}

pub(crate) fn response_options(
    team_id: Uuid,
    configs: &[ProjectConfig],
) -> GithubCommandResponseOptions {
    GithubCommandResponseOptions {
        team_id,
        projects: configs
            .iter()
            .map(|config| GithubProjectOption {
                project_config_id: config.id,
                display_name: match config.name.trim() {
                    "" => config.external_project_id.clone(),
                    _ => config.name.clone(),
                },
            })
            .collect(),
    }
}
