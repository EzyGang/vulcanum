use std::io;
use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use vulcanum_shared::api::app::model_providers::{PollDeviceFlowResponse, StartDeviceFlowRequest};
use vulcanum_shared::state::app as app_state;

use crate::commands::app::settings::runtime::SettingsRuntime;
use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::escape_terminal;

const OPENAI_PROVIDER_KEY: &str = "openai";
const OPENAI_DEVICE_PROVIDER: &str = "openai_chatgpt";

pub(crate) async fn connect_openai(
    name: Option<String>,
    no_browser: bool,
    team: Option<Uuid>,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut settings = SettingsRuntime::real();
    connect_openai_with(name.as_deref(), no_browser, team, &mut app, &mut settings).await
}

pub(super) async fn connect_openai_with(
    name: Option<&str>,
    no_browser: bool,
    team_override: Option<Uuid>,
    app: &mut AppRuntime<'_>,
    settings: &mut SettingsRuntime,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, team_override).await?;
    let request = StartDeviceFlowRequest {
        provider_key: OPENAI_PROVIDER_KEY.to_owned(),
        device_provider: OPENAI_DEVICE_PROVIDER.to_owned(),
        display_name: name.unwrap_or_default().to_owned(),
    };
    let flow = context
        .client
        .start_model_provider_device_flow(team.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Start OpenAI device flow", error))?;
    writeln!(
        app.stdout,
        "Open {} and enter code {}.",
        escape_terminal(&flow.verification_uri),
        escape_terminal(&flow.user_code)
    )?;
    if !no_browser && (settings.open_browser)(&flow.verification_uri).is_err() {
        writeln!(
            settings.stderr,
            "Warning: could not open the browser; continue with the printed URL and code."
        )?;
    }

    let now = (settings.now)();
    let initial_delay = Duration::from_secs(flow.interval_seconds.max(0) as u64);
    (settings.sleep)(bounded_delay(now, initial_delay, flow.expires_at)).await;
    loop {
        if (settings.now)() >= flow.expires_at {
            anyhow::bail!("OpenAI device authorization expired");
        }
        let response = context
            .client
            .poll_model_provider_device_flow(
                team.id,
                flow.attempt_id,
                &context.session.access_token,
            )
            .await
            .map_err(|error| handle_authenticated_error("Poll OpenAI device flow", error))?;
        match response {
            PollDeviceFlowResponse::Connected { provider } => {
                writeln!(
                    app.stdout,
                    "Connected OpenAI as {} ({}) for team {}.",
                    escape_terminal(&provider.display_name),
                    provider.id,
                    team.id
                )?;
                return Ok(());
            }
            PollDeviceFlowResponse::Pending { next_poll_at } => {
                let now = (settings.now)();
                if now >= flow.expires_at {
                    anyhow::bail!("OpenAI device authorization expired");
                }
                let delay = poll_delay(now, next_poll_at);
                (settings.sleep)(bounded_delay(now, delay, flow.expires_at)).await;
            }
        }
    }
}

fn bounded_delay(now: DateTime<Utc>, requested: Duration, expires_at: DateTime<Utc>) -> Duration {
    let until_expiry = expires_at
        .signed_duration_since(now)
        .to_std()
        .unwrap_or(Duration::ZERO);
    requested.min(until_expiry)
}

fn poll_delay(now: DateTime<Utc>, next_poll_at: DateTime<Utc>) -> Duration {
    next_poll_at
        .signed_duration_since(now)
        .to_std()
        .unwrap_or(Duration::ZERO)
}
