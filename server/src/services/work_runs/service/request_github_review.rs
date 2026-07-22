use uuid::Uuid;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_pr_url;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct GithubReviewRequest<'a> {
    pub delivery_id: &'a str,
    pub installation_id: i64,
    pub sender_id: &'a str,
    pub repo_full_name: &'a str,
    pub pr_number: i64,
    pub pr_title: &'a str,
    pub project_selector: Option<&'a str>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ReviewProjectOption {
    pub project_config_id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ReviewResponseOptions {
    pub team_id: Uuid,
    pub projects: Vec<ReviewProjectOption>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum GithubReviewRequestOutcome {
    Spawned,
    AlreadyActive,
    Unauthorized,
    UnknownInstallation,
    NoMatchingProject { team_id: Uuid },
    ReviewDisabled(ReviewResponseOptions),
    ProjectSelectionRequired(ReviewResponseOptions),
    InvalidProjectSelection(ReviewResponseOptions),
}

impl WorkRunsService {
    pub(crate) async fn request_github_review(
        &self,
        request: GithubReviewRequest<'_>,
    ) -> Result<GithubReviewRequestOutcome, WorkRunsError> {
        let team_id = match self
            .github
            .repo
            .find_team_id_by_github_installation(&self.db, request.installation_id)
            .await?
        {
            Some(team_id) => team_id,
            None => return Ok(GithubReviewRequestOutcome::UnknownInstallation),
        };
        let authorized = self
            .project_configs
            .teams
            .is_provider_identity_member(team_id, "github", request.sender_id)
            .await?;
        if !authorized {
            return Ok(GithubReviewRequestOutcome::Unauthorized);
        }

        let configs = self
            .project_configs
            .list_enabled_for_github_repo(request.installation_id, request.repo_full_name)
            .await?;
        if configs.is_empty() {
            return Ok(GithubReviewRequestOutcome::NoMatchingProject { team_id });
        }

        let mut enabled = Vec::new();
        let mut disabled = Vec::new();
        for config in configs {
            let settings = self.project_configs.effective_settings(&config).await?;
            match settings.review_enabled {
                true => enabled.push(config),
                false => disabled.push(config),
            }
        }
        let options = response_options(team_id, &enabled);
        let selected = match select_project(request.project_selector, &enabled, &disabled) {
            ProjectSelection::Selected(config) => config,
            ProjectSelection::Disabled => {
                return Ok(GithubReviewRequestOutcome::ReviewDisabled(options));
            }
            ProjectSelection::Required => {
                return Ok(GithubReviewRequestOutcome::ProjectSelectionRequired(
                    options,
                ));
            }
            ProjectSelection::Invalid => {
                return Ok(GithubReviewRequestOutcome::InvalidProjectSelection(options));
            }
        };

        let inserted = self
            .work_runs_repo
            .insert_work_run_if_not_active(
                &self.db,
                InsertWorkRunParams {
                    team_id,
                    external_task_ref: format!(
                        "github-pr:{}#{}",
                        request.repo_full_name.to_ascii_lowercase(),
                        request.pr_number,
                    ),
                    task_title: Some(request.pr_title.to_owned()),
                    task_slug: Some(format!("{}#{}", request.repo_full_name, request.pr_number)),
                    project_config_id: selected.id,
                    repo_full_names: vec![request.repo_full_name.to_owned()],
                    status: WorkRunStatus::Pending,
                    work_type: WorkRunType::PullRequestReview,
                    parent_work_run_id: None,
                    review_target_pr_url: Some(github_pr_url(
                        request.repo_full_name,
                        request.pr_number,
                    )),
                    review_target_repo_full_name: Some(request.repo_full_name.to_owned()),
                    github_installation_id: Some(request.installation_id),
                    github_delivery_id: Some(request.delivery_id.to_owned()),
                },
            )
            .await?;

        match inserted {
            true => Ok(GithubReviewRequestOutcome::Spawned),
            false => Ok(GithubReviewRequestOutcome::AlreadyActive),
        }
    }
}

enum ProjectSelection<'a> {
    Selected(&'a ProjectConfig),
    Disabled,
    Required,
    Invalid,
}

fn select_project<'a>(
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

fn response_options(team_id: Uuid, configs: &[ProjectConfig]) -> ReviewResponseOptions {
    ReviewResponseOptions {
        team_id,
        projects: configs
            .iter()
            .map(|config| ReviewProjectOption {
                project_config_id: config.id,
                display_name: match config.name.trim() {
                    "" => config.external_project_id.clone(),
                    _ => config.name.clone(),
                },
            })
            .collect(),
    }
}
