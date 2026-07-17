use std::io;

use uuid::Uuid;
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
    let workers = context
        .client
        .list_workers(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("List workers", error))?;

    let team_name = escape_terminal(&team.name);
    if workers.is_empty() {
        writeln!(
            runtime.stdout,
            "No workers found for team {team_name} ({}).",
            team.id
        )?;
        return Ok(());
    }

    let rows = workers
        .into_iter()
        .map(|worker| {
            vec![
                worker.id.to_string(),
                worker.name,
                worker.status,
                worker
                    .last_seen
                    .map_or_else(|| "never".to_owned(), |seen| seen.to_rfc3339()),
                format!("{}/{}", worker.active_jobs, worker.max_concurrent_jobs),
            ]
        })
        .collect();
    let table = render_table(&["ID", "NAME", "STATUS", "LAST SEEN", "LOAD"], rows);
    writeln!(
        runtime.stdout,
        "Workers — {team_name} ({})\n{table}",
        team.id
    )?;
    Ok(())
}
