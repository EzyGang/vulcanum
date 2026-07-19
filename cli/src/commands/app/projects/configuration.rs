use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::projects::UpdateProjectRequest;
use vulcanum_shared::api::app::task_board::TaskBoardResponse;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::board::support::{find_column, load_board, load_project};
use crate::commands::app::{handle_authenticated_error, AppRuntime};
use crate::console::escape_terminal;

pub(crate) struct ColumnsOptions {
    pub(crate) project_id: Uuid,
    pub(crate) pickup: Option<String>,
    pub(crate) in_progress: Option<String>,
    pub(crate) in_review: Option<String>,
    pub(crate) done: Option<String>,
    pub(crate) team: Option<Uuid>,
}

pub async fn set_automation(
    project_id: Uuid,
    enabled: bool,
    team: Option<Uuid>,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    set_automation_with(project_id, enabled, team, &mut runtime).await
}

pub async fn set_columns(options: ColumnsOptions) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    set_columns_with(options, &mut runtime).await
}

pub(super) async fn set_automation_with(
    project_id: Uuid,
    enabled: bool,
    team: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let (context, project, team_id, _) = load_project(project_id, team, runtime).await?;
    if enabled && project.repo_full_names.is_empty() {
        anyhow::bail!(
            "Project automation requires an attached repository. Attach one with \
             `vulcanum projects repos set {} --repo OWNER/NAME`, then retry.",
            project.id
        );
    }
    let request = UpdateProjectRequest {
        enabled: Some(enabled),
        ..UpdateProjectRequest::default()
    };
    let updated = context
        .client
        .update_project(team_id, project_id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Update project automation", error))?;
    writeln!(
        runtime.stdout,
        "Automation {} for project {} ({}).",
        if updated.enabled {
            "enabled"
        } else {
            "disabled"
        },
        escape_terminal(&project.name),
        project.id
    )?;
    Ok(())
}

pub(super) async fn set_columns_with(
    options: ColumnsOptions,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    if options.pickup.is_none()
        && options.in_progress.is_none()
        && options.in_review.is_none()
        && options.done.is_none()
    {
        anyhow::bail!(
            "Specify --pickup, --in-progress, --in-review, or --done with a board column."
        );
    }
    let loaded = load_board(options.project_id, options.team, runtime).await?;
    let request = UpdateProjectRequest {
        pickup_column: resolve_column(&loaded.board, options.pickup.as_deref())?,
        progress_column: resolve_column(&loaded.board, options.in_progress.as_deref())?,
        review_column: resolve_column(&loaded.board, options.in_review.as_deref())?,
        done_column: resolve_column(&loaded.board, options.done.as_deref())?,
        ..UpdateProjectRequest::default()
    };
    let updated = loaded
        .context
        .client
        .update_project(
            loaded.team_id,
            options.project_id,
            &request,
            &loaded.context.session.access_token,
        )
        .await
        .map_err(|error| handle_authenticated_error("Update project columns", error))?;
    writeln!(
        runtime.stdout,
        "Updated workflow columns for project {} ({}): pickup={}, in-progress={}, in-review={}, done={}.",
        escape_terminal(&updated.name),
        updated.id,
        render_mark(&updated.pickup_column),
        render_mark(&updated.progress_column),
        render_mark(&updated.review_column),
        render_mark(&updated.done_column)
    )?;
    Ok(())
}

fn resolve_column(
    board: &TaskBoardResponse,
    selector: Option<&str>,
) -> anyhow::Result<Option<String>> {
    selector
        .map(|value| find_column(board, value).map(|column| column.slug.clone()))
        .transpose()
}

fn render_mark(value: &str) -> String {
    if value.is_empty() {
        "unset".to_owned()
    } else {
        escape_terminal(value).into_owned()
    }
}
