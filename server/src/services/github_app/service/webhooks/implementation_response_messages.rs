use uuid::Uuid;

use crate::services::github_app::service::webhooks::responses::{
    markdown_escape, GithubResponseTarget,
};
use crate::services::work_runs::service::github_commands::{
    GithubCommandResponseOptions, GithubProjectOption,
};
use crate::services::work_runs::service::request_github_implementation::ImplementationCommandError;

#[derive(Clone, Copy)]
pub(super) struct ImplementationResponseContext<'a> {
    pub app_slug: &'a str,
    pub target: GithubResponseTarget<'a>,
    pub request_body: Option<&'a str>,
    pub ticket_selector: Option<&'a str>,
}

pub(super) fn ticket_action(created: bool) -> &'static str {
    match created {
        true => "created",
        false => "reused",
    }
}

pub(super) fn malformed_message(app_slug: &str, error: ImplementationCommandError) -> String {
    let reason = match error {
        ImplementationCommandError::Malformed => {
            "The implementation command is malformed or its request is empty."
        }
        ImplementationCommandError::Ambiguous => {
            "The comment contains multiple conflicting Vulcanum commands."
        }
    };
    format!(
        "{reason}\n\nUse `@{app_slug} implement [project:<project-config-uuid>] [ticket:<external-task-ref>] <request>`. The request is required and may span multiple lines."
    )
}

pub(super) fn ticket_selection_message(
    heading: &str,
    context: ImplementationResponseContext<'_>,
    project_config_id: Uuid,
    external_task_refs: &[String],
) -> String {
    let request_body = context.request_body.unwrap_or("<request>");
    if external_task_refs.is_empty() {
        return format!("{heading}\n\nNo mapped implementation ticket is available.");
    }
    let choices = external_task_refs
        .iter()
        .map(|external_task_ref| {
            let command = format!(
                "@{} implement project:{project_config_id} ticket:{external_task_ref} {request_body}",
                context.app_slug,
            )
            .replace('\n', "\n    ");
            format!(
                "{}\n\n    {command}",
                markdown_escape(external_task_ref),
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");
    format!("{heading}\n\n{choices}")
}

pub(super) fn project_choices(
    heading: &str,
    context: ImplementationResponseContext<'_>,
    options: &GithubCommandResponseOptions,
) -> String {
    let request_body = context.request_body.unwrap_or("<request>");
    if options.projects.is_empty() {
        return format!("{heading}\n\nNo enabled project is available.");
    }
    let choices = options
        .projects
        .iter()
        .map(|project| implementation_choice(context, request_body, project))
        .collect::<Vec<String>>()
        .join("\n\n");
    format!("{heading}\n\n{choices}")
}

fn implementation_choice(
    context: ImplementationResponseContext<'_>,
    request_body: &str,
    project: &GithubProjectOption,
) -> String {
    let ticket_selector = context
        .ticket_selector
        .map(|selector| format!(" ticket:{selector}"))
        .unwrap_or_default();
    let command = format!(
        "@{} implement project:{}{ticket_selector} {request_body}",
        context.app_slug, project.project_config_id,
    )
    .replace('\n', "\n    ");
    format!(
        "{}\n\n    {command}",
        markdown_escape(&project.display_name),
    )
}
