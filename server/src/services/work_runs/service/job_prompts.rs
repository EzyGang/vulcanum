use vulcanum_shared::api::wire::JobRepo;

use crate::models::project_configs::model::JobConfigFields;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::model::{GithubImplementationFollowupContext, WorkRun, WorkRunType};
use crate::services::poller::prompts::{
    ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION, IMPLEMENTATION_FOLLOWUP_GITHUB_INSTRUCTION,
    REVIEW_GITHUB_INSTRUCTION,
};
use crate::services::poller::service::repo_layout;
use crate::services::poller::template::{render_template, TemplateVars};
use crate::util::github::{github_pr_url, github_repo_url};

#[must_use]
pub(crate) fn render_prompt_text(
    run: &WorkRun,
    cfg: &JobConfigFields,
    task: &IntegrationTask,
    repos: &[JobRepo],
    implementation_followup: Option<&GithubImplementationFollowupContext>,
) -> String {
    match run.work_type {
        WorkRunType::Implementation => match implementation_followup {
            Some(context) => render_implementation_followup_prompt(task, context),
            None => render_implementation_prompt(cfg, task, repos),
        },
        WorkRunType::PullRequestReview => render_review_prompt(run, cfg, task),
    }
}

#[must_use]
pub(crate) fn render_implementation_prompt(
    cfg: &JobConfigFields,
    task: &IntegrationTask,
    repos: &[JobRepo],
) -> String {
    let repo_urls = repos
        .iter()
        .map(|repo| repo.url.as_str())
        .collect::<Vec<&str>>()
        .join("\n");
    let repo_full_names = repos
        .iter()
        .map(|repo| repo.full_name.clone())
        .collect::<Vec<String>>();
    let repo_names = repo_full_names.join("\n");
    let repo_layout = repo_layout(&repo_full_names);
    let repo_url = repos.first().map(|repo| repo.url.as_str()).unwrap_or("");
    let mut prompt_text = render_template(
        &cfg.prompt_template,
        &TemplateVars {
            task_title: &task.title,
            task_body: task.description.as_deref().unwrap_or(""),
            repo_url,
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &repo_layout,
            review_target_pr_url: "",
        },
    );

    prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
    if !repos.is_empty() {
        prompt_text.push_str(GITHUB_INSTRUCTION);
    }
    prompt_text
}

#[must_use]
pub(crate) fn render_implementation_followup_prompt(
    task: &IntegrationTask,
    context: &GithubImplementationFollowupContext,
) -> String {
    let pr_url = github_pr_url(&context.repo_full_name, context.pr_number);
    let mut prompt_text = format!(
        "# Pull request implementation follow-up\n\n\
         ## Resolved tracker task\n\n\
         Title:\n{}\n\n\
         Full current description:\n{}\n\n\
         ## Source pull request\n\n\
         Repository: {}\n\
         Pull request: {pr_url}\n\n\
         ## Exact GitHub follow-up request\n\n\
         {}\n\n\
         ## Instructions\n\n\
         Treat the tracker title and full description above as the durable current task contract. \
         Inspect the current pull request, its branch, and all repository instructions before editing. \
         Implement only the exact follow-up request, validate the resulting change, and preserve the existing pull request workflow.",
        task.title,
        task.description.as_deref().unwrap_or(""),
        context.repo_full_name,
        context.request_body,
    );
    prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
    prompt_text.push_str(IMPLEMENTATION_FOLLOWUP_GITHUB_INSTRUCTION);
    prompt_text
}

#[must_use]
fn render_review_prompt(run: &WorkRun, cfg: &JobConfigFields, task: &IntegrationTask) -> String {
    let repo_names = match run.review_target_repo_full_name.as_deref() {
        Some(repo) => repo.to_owned(),
        None => cfg.repo_full_names.join("\n"),
    };
    let repo_urls = match run.review_target_repo_full_name.as_deref() {
        Some(repo) => github_repo_url(repo),
        None => cfg.repo_urls.join("\n"),
    };
    let repo_full_names = match run.review_target_repo_full_name.as_ref() {
        Some(repo) => vec![repo.clone()],
        None => cfg.repo_full_names.clone(),
    };
    let repo_layout = repo_layout(&repo_full_names);

    let mut prompt_text = render_template(
        &cfg.review_prompt_template,
        &TemplateVars {
            task_title: &task.title,
            task_body: task.description.as_deref().unwrap_or(""),
            repo_url: &repo_urls,
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &repo_layout,
            review_target_pr_url: run.review_target_pr_url.as_deref().unwrap_or(""),
        },
    );
    prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
    if !repo_full_names.is_empty() {
        prompt_text.push_str(REVIEW_GITHUB_INSTRUCTION);
    }
    prompt_text
}
