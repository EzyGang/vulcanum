use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::util::serde::deserialize_nullable_string;

pub const DEFAULT_REVIEW_PROMPT_TEMPLATE: &str = r#"Review this pull request for the linked task.

Task title:
{{task_title}}

Task body:
{{task_body}}

Focus pull request:
{{review_target_pr_url}}

Repository:
{{repo_names}}

Follow the repository AGENTS.md instructions. Review code quality, correctness, maintainability, and project conventions. Do not edit files, commit, push, or create pull requests. Post exactly one GitHub pull request review comment using gh. Use comment-only review, not approve or request changes. Include this marker in the review body: {{review_marker}}. If the marker already exists on the pull request, do not post another review.

The review body must use exactly these Markdown sections in this order:
## CRITICAL
- List defects that make the implementation unsafe, incorrect, or unusable. Use "- None" if empty.

## WARNINGS
- List defects that should be fixed before merging. Use "- None" if empty.

## SUGGESTIONS
- List optional improvements. Use "- None" if empty.

When done, call finish_run with status completed, review_url if available, review_body, and review_already_exists."#;

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
    pub review_pickup_column: String,
    pub review_max_turns: i32,
    pub review_prompt_template: String,
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
    pub review_pickup_column: Option<String>,
    #[serde(default)]
    pub review_max_turns: Option<i32>,
    #[serde(default)]
    pub review_prompt_template: Option<String>,
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
