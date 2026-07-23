use uuid::Uuid;
use vulcanum_shared::api::wire::{JobRepo, RefreshGithubTokenResponse};

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn refresh_github_token(
        &self,
        id: Uuid,
        worker_id: Uuid,
    ) -> Result<RefreshGithubTokenResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        if !matches!(run.status, WorkRunStatus::Running) {
            return Err(WorkRunsError::InvalidStatusTransition);
        }

        let repos = self.github_repos_for_work_run(&run).await?;
        self.mint_github_token_for_repos(&run, &repos).await
    }

    pub(crate) async fn github_repos_for_work_run(
        &self,
        run: &WorkRun,
    ) -> Result<Vec<JobRepo>, WorkRunsError> {
        self.work_runs_repo.list_repos(&self.db, run.id).await
    }

    pub(crate) async fn mint_github_token_for_repos(
        &self,
        run: &WorkRun,
        repos: &[JobRepo],
    ) -> Result<RefreshGithubTokenResponse, WorkRunsError> {
        let repo_full_names = repos
            .iter()
            .map(|repo| repo.full_name.clone())
            .collect::<Vec<String>>();

        if repo_full_names.is_empty() {
            return Ok(RefreshGithubTokenResponse {
                github_token: None,
                github_token_expires_at: None,
            });
        }

        let token_result = match run.github_installation_id {
            Some(installation_id) => {
                self.github
                    .generate_installation_token_for_installation(
                        run.team_id,
                        installation_id,
                        &repo_full_names,
                    )
                    .await
            }
            None => {
                self.github
                    .generate_installation_token_for_repos(run.team_id, &repo_full_names)
                    .await
            }
        };
        let token = match token_result {
            Ok(token) => token,
            Err(e) => {
                tracing::error!(
                    work_run_id = %run.id,
                    error = %e,
                    "failed to mint github installation token"
                );
                return Err(e.into());
            }
        };

        Ok(RefreshGithubTokenResponse {
            github_token: Some(token.token),
            github_token_expires_at: Some(token.expires_at),
        })
    }
}
