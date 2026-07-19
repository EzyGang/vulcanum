use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use vulcanum_shared::api::wire::AgentBackend;

use crate::util::serde::deserialize_nullable_string;

pub const DEFAULT_PROMPT_TEMPLATE: &str = r#"Implement the linked task.

Before editing, inspect the repository instructions and project manifests. Spawn a focused setup subagent when dependencies, generated files, or environment preparation are not already clear; have it follow the relevant AGENTS.md files and run the installation/setup commands the project needs.

Task title:
{{task_title}}

Task body:
```text
{{task_body}}
```

Repositories:
{{repo_urls}}

Follow the repository instructions, keep changes focused on the task, and keep the final response concise."#;

pub const DEFAULT_REVIEW_PROMPT_TEMPLATE: &str = r#"Review this pull request for the linked task.

Before judging the implementation, inspect the repository instructions and project manifests. Follow every AGENTS.md file that applies to the changed directories. Spawn focused read-only subagents when they would help check correctness, tests, security, or project conventions; they must not edit files.

Review the solution for correctness, maintainability, and project fit. Make sure the implementation is elegant, avoids duplication, and has been formatted and validated with the repository commands that apply to the changed code. During the review phase, do not edit files, commit, push, or create pull requests. Keep the final response concise and focused on actionable findings.

Post exactly one GitHub pull request review comment using gh. Use comment-only review, not approve or request changes. If a suitable review already exists for the current PR head commit, do not post a duplicate review. If the PR has new commits after the existing review, post a new review.

The review body must use exactly these Markdown sections in this order:
## CRITICAL
- List defects that make the implementation unsafe, incorrect, or unusable. Use "- None" if empty.

## WARNINGS
- List defects that should be fixed before merging, including missing or failing formatter, validation, or test commands. This includes serious violations of AGENTS.md guidelines. Use "- None" if empty.

## SUGGESTIONS
- List optional improvements. Use "- None" if empty.

When done, call finish_run with status completed, review_url if available, and review_body.

Task title:
{{task_title}}

Task body:
{{task_body}}

Focus pull request:
{{review_target_pr_url}}

Repository:
{{repo_names}}"#;

#[derive(Debug, Clone, Serialize)]
pub struct TeamDefaultsResponse {
    pub review_prompt_template: &'static str,
    pub max_in_progress_tasks: i32,
    pub prompt_template: &'static str,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub personal_user_id: Option<String>,
    pub prompt_template: String,
    pub agents_md: String,
    pub primary_model_provider_key: Option<String>,
    pub primary_model_id: Option<String>,
    pub small_model_provider_key: Option<String>,
    pub small_model_id: Option<String>,
    pub review_enabled: bool,
    pub review_max_turns: i32,
    pub review_prompt_template: String,
    pub max_in_progress_tasks: i32,
    pub agent_backend: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TeamMemberInfo {
    pub team_id: Uuid,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub agents_md: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub primary_model_provider_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub primary_model_id: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub small_model_provider_key: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_string")]
    pub small_model_id: Option<Option<String>>,
    #[serde(default)]
    pub review_enabled: Option<bool>,
    #[serde(default)]
    pub review_max_turns: Option<i32>,
    #[serde(default)]
    pub review_prompt_template: Option<String>,
    #[serde(default)]
    pub max_in_progress_tasks: Option<i32>,
    #[serde(default)]
    pub agent_backend: Option<AgentBackend>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTeamInviteResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamInvitePreviewResponse {
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptTeamInviteResponse {
    pub team_id: Uuid,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentity {
    pub user_id: String,
    pub provider_user_id: String,
    pub provider_login: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ProviderIdentity {
    pub provider: String,
    pub provider_user_id: String,
    pub provider_login: String,
    pub provider_verified_at: Option<DateTime<Utc>>,
}
