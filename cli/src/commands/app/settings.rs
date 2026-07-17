use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::{AppModelProvider, AppTeam, GithubAppInstallation};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::constants::DEFAULT_TEAM_ID;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::{
    authenticated_context, handle_authenticated_error, load_required_session, resolve_team,
    sanitize_request_error, AppRuntime,
};
use crate::console::{escape_terminal, redact_url, render_table};

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

pub async fn set_team(team_id: Uuid) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    set_team_with(team_id, &mut runtime).await
}

pub async fn clear_team() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    clear_team_with(&mut runtime).await
}

pub(super) async fn list_with(
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let access_token = &context.session.access_token;
    let team_id = team.id;
    let trackers_request = async {
        context
            .client
            .list_task_trackers(team_id, access_token)
            .await
            .map_err(|error| handle_authenticated_error("List task trackers", error))
    };
    let providers_request = async {
        context
            .client
            .list_model_providers(team_id, access_token)
            .await
            .map_err(|error| handle_authenticated_error("List model providers", error))
    };
    let github_request = async {
        context
            .client
            .get_github_app_installation(team_id, access_token)
            .await
            .map_err(|error| handle_authenticated_error("Get GitHub App installation", error))
    };
    let (trackers, providers, installation) =
        tokio::try_join!(trackers_request, providers_request, github_request)?;

    let output = render_settings(&team, trackers, providers, installation);
    write!(runtime.stdout, "{output}")?;
    Ok(())
}

pub(super) async fn set_team_with(
    team_id: Uuid,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let mut context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, Some(team_id)).await?;
    context.session.team_id = Some(team.id);
    (runtime.save_session)(&context.session)?;
    writeln!(
        runtime.stdout,
        "Pinned team {} ({}).",
        escape_terminal(&team.name),
        team.id
    )?;
    Ok(())
}

pub(super) async fn clear_team_with(runtime: &mut AppRuntime<'_>) -> anyhow::Result<()> {
    let mut session = load_required_session(runtime)?;
    let mode = ApiClient::new(&session.instance_url)
        .auth_mode()
        .await
        .map_err(|error| sanitize_request_error("Get authentication mode", error))?;
    match mode.is_single_user {
        true => {
            session.team_id = Some(DEFAULT_TEAM_ID);
            (runtime.save_session)(&session)?;
            writeln!(
                runtime.stdout,
                "Reset the pinned team to {DEFAULT_TEAM_ID}."
            )?;
        }
        false => {
            session.team_id = None;
            (runtime.save_session)(&session)?;
            writeln!(runtime.stdout, "Cleared the pinned team.")?;
        }
    }
    Ok(())
}

fn render_settings(
    team: &AppTeam,
    trackers: Vec<vulcanum_shared::api::app::TaskTracker>,
    providers: Vec<AppModelProvider>,
    installation: Option<GithubAppInstallation>,
) -> String {
    let team_name = escape_terminal(&team.name);
    let primary_provider = provider_label(team.primary_model_provider_key.as_deref(), &providers);
    let small_provider = provider_label(team.small_model_provider_key.as_deref(), &providers);
    let primary_model = option_label(team.primary_model_id.as_deref());
    let small_model = option_label(team.small_model_id.as_deref());
    let mut output = format!(
        "Settings — {team_name} ({})\n\nModel Selection\nPrimary provider: {primary_provider}\nPrimary model: {primary_model}\nSmall provider: {small_provider}\nSmall model: {small_model}\n\nTask Trackers\n",
        team.id
    );

    if trackers.is_empty() {
        output.push_str("No task trackers configured.\n");
    } else {
        let rows = trackers
            .into_iter()
            .map(|tracker| {
                vec![
                    tracker.name,
                    tracker.provider_type,
                    redact_url(&tracker.instance_url),
                ]
            })
            .collect();
        output.push_str(&render_table(&["NAME", "TYPE", "INSTANCE URL"], rows));
        output.push('\n');
    }

    output.push_str("\nModel Providers\n");
    if providers.is_empty() {
        output.push_str("No model providers configured.\n");
    } else {
        let rows = providers
            .into_iter()
            .map(|provider| {
                let credential_fields = match provider.credential_fields.is_empty() {
                    true => "-".to_owned(),
                    false => provider.credential_fields.join(", "),
                };
                let oauth_account = provider
                    .oauth
                    .and_then(|oauth| oauth.email.or(oauth.account_id))
                    .unwrap_or_else(|| "-".to_owned());
                vec![
                    provider.display_name,
                    provider.provider_key,
                    provider.auth_type,
                    credential_fields,
                    oauth_account,
                ]
            })
            .collect();
        output.push_str(&render_table(
            &[
                "NAME",
                "PROVIDER",
                "AUTH",
                "CREDENTIAL FIELDS",
                "OAUTH ACCOUNT",
            ],
            rows,
        ));
        output.push('\n');
    }

    output.push_str("\nGitHub App\n");
    match installation {
        Some(installation) => {
            output.push_str("Status: connected\nAccount: ");
            output.push_str(&escape_terminal(&installation.account_login));
            output.push('\n');
        }
        None => output.push_str("Status: disconnected\n"),
    }
    output
}

fn provider_label(key: Option<&str>, providers: &[AppModelProvider]) -> String {
    match key {
        Some(key) => match providers
            .iter()
            .find(|provider| provider.provider_key == key)
        {
            Some(provider) => format!(
                "{} ({})",
                escape_terminal(&provider.display_name),
                escape_terminal(key)
            ),
            None => escape_terminal(key).into_owned(),
        },
        None => "-".to_owned(),
    }
}

fn option_label(value: Option<&str>) -> String {
    value.map_or_else(
        || "-".to_owned(),
        |value| escape_terminal(value).into_owned(),
    )
}
