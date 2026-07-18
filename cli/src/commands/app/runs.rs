use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::work_runs::WorkRunListItem;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::{escape_terminal, render_table};

pub async fn list(team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    list_with(team, &mut runtime).await
}

pub(super) async fn list_with(
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let runs = context
        .client
        .list_work_runs(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("List work runs", error))?;
    let team_name = escape_terminal(&team.name);
    if runs.is_empty() {
        writeln!(
            runtime.stdout,
            "No work runs found for team {team_name} ({}).",
            team.id
        )?;
        return Ok(());
    }

    let rows = runs.into_iter().map(render_row).collect();
    let table = render_table(
        &[
            "ID",
            "TICKET",
            "TITLE",
            "TYPE",
            "STATUS",
            "TOKENS (I/O/CACHE)",
            "MODEL",
            "DURATION",
            "CREATED",
        ],
        rows,
    );
    writeln!(
        runtime.stdout,
        "Work Runs — {team_name} ({})\n{table}",
        team.id
    )?;
    Ok(())
}

fn render_row(run: WorkRunListItem) -> Vec<String> {
    vec![
        run.id.to_string(),
        escape_terminal(&run.external_task_ref).into_owned(),
        run.task_title.as_deref().map_or_else(
            || "-".to_owned(),
            |title| escape_terminal(title).into_owned(),
        ),
        run.work_type.to_string(),
        run.status.to_string(),
        token_usage(&run),
        run.model_used.as_deref().map_or_else(
            || "-".to_owned(),
            |model| escape_terminal(model).into_owned(),
        ),
        run.duration_ms
            .map_or_else(|| "-".to_owned(), |duration| format!("{duration} ms")),
        run.created_at.to_rfc3339(),
    ]
}

fn token_usage(run: &WorkRunListItem) -> String {
    let granular = [
        run.input_tokens,
        run.output_tokens,
        run.cache_read_tokens,
        run.cache_write_tokens,
    ];
    let has_granular = granular.iter().any(Option::is_some);
    let total = match run.tokens_used {
        Some(total) => Some(total),
        None if has_granular => Some(
            granular
                .into_iter()
                .flatten()
                .fold(0_i64, i64::saturating_add),
        ),
        None => None,
    };
    match (total, has_granular) {
        (Some(total), true) => {
            let cache = run
                .cache_read_tokens
                .unwrap_or_default()
                .saturating_add(run.cache_write_tokens.unwrap_or_default());
            format!(
                "{total} ({}/{}/{cache})",
                run.input_tokens.unwrap_or_default(),
                run.output_tokens.unwrap_or_default()
            )
        }
        (Some(total), false) => total.to_string(),
        (None, _) => "-".to_owned(),
    }
}
