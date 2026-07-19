use std::io;

use serde_json::Value;
use uuid::Uuid;
use vulcanum_shared::api::app::model_providers::{
    CreateModelProviderRequest, ModelProviderAuthType, UpdateModelProviderRequest,
};
use vulcanum_shared::state::app as app_state;

use crate::commands::app::args::DirectModelProviderAuth;
use crate::commands::app::settings::credentials::model_provider_credentials;
use crate::commands::app::settings::runtime::SettingsRuntime;
use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::escape_terminal;

pub(crate) struct AddOptions {
    pub(crate) provider_key: String,
    pub(crate) name: Option<String>,
    pub(crate) auth: DirectModelProviderAuth,
    pub(crate) credentials_stdin: bool,
    pub(crate) team: Option<Uuid>,
}

pub(crate) struct UpdateOptions {
    pub(crate) id: Uuid,
    pub(crate) name: Option<String>,
    pub(crate) auth: Option<DirectModelProviderAuth>,
    pub(crate) credentials_stdin: bool,
    pub(crate) prompt_credentials: bool,
    pub(crate) team: Option<Uuid>,
}

pub(crate) async fn add(options: AddOptions) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut settings = SettingsRuntime::real();
    add_with(options, &mut app, &mut settings).await
}

pub(crate) async fn update(options: UpdateOptions) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut settings = SettingsRuntime::real();
    update_with(options, &mut app, &mut settings).await
}

pub(crate) async fn remove(id: Uuid, team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    remove_with(id, team, &mut app).await
}

pub(super) async fn add_with(
    options: AddOptions,
    app: &mut AppRuntime<'_>,
    settings: &mut SettingsRuntime,
) -> anyhow::Result<()> {
    if options.auth == DirectModelProviderAuth::None && options.credentials_stdin {
        anyhow::bail!("auth=none does not accept credential flags");
    }
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, options.team).await?;
    let credentials = match options.auth {
        DirectModelProviderAuth::None => Value::Null,
        DirectModelProviderAuth::ApiKey => {
            let catalog = match options.credentials_stdin {
                true => None,
                false => Some(
                    context
                        .client
                        .get_model_catalog(team.id, &context.session.access_token)
                        .await
                        .map_err(|error| handle_authenticated_error("Get model catalog", error))?,
                ),
            };
            let provider = catalog.as_ref().and_then(|catalog| {
                catalog
                    .providers
                    .iter()
                    .find(|provider| provider.id == options.provider_key)
            });
            model_provider_credentials(options.credentials_stdin, provider, settings)?
        }
    };
    let request = CreateModelProviderRequest {
        provider_key: options.provider_key,
        display_name: options.name.unwrap_or_default(),
        auth_type: options.auth.into(),
        credentials,
    };
    let provider = context
        .client
        .create_model_provider(team.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Add model provider", error))?;
    writeln!(
        app.stdout,
        "Added model provider {} ({}) for team {}.",
        escape_terminal(&provider.display_name),
        provider.id,
        team.id
    )?;
    Ok(())
}

pub(super) async fn update_with(
    options: UpdateOptions,
    app: &mut AppRuntime<'_>,
    settings: &mut SettingsRuntime,
) -> anyhow::Result<()> {
    if options.name.is_none()
        && options.auth.is_none()
        && !options.credentials_stdin
        && !options.prompt_credentials
    {
        anyhow::bail!("Model provider update requires at least one change");
    }
    let replacing_credentials = options.credentials_stdin || options.prompt_credentials;
    match options.auth {
        Some(DirectModelProviderAuth::None) if replacing_credentials => {
            anyhow::bail!("auth=none does not accept credential flags");
        }
        Some(DirectModelProviderAuth::ApiKey) if !replacing_credentials => {
            anyhow::bail!("Changing to auth=api-key requires a credential flag");
        }
        _ => (),
    }

    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, options.team).await?;
    let providers = context
        .client
        .list_model_providers(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("List model providers", error))?;
    let provider = providers
        .iter()
        .find(|provider| provider.id == options.id)
        .ok_or_else(|| anyhow::anyhow!("Model provider {} was not found", options.id))?;
    if replacing_credentials
        && provider.auth_type != ModelProviderAuthType::ApiKey
        && options.auth != Some(DirectModelProviderAuth::ApiKey)
    {
        anyhow::bail!("Replacing credentials on this provider requires --auth api-key");
    }

    let credentials = match replacing_credentials {
        true => {
            let catalog = match options.credentials_stdin {
                true => None,
                false => Some(
                    context
                        .client
                        .get_model_catalog(team.id, &context.session.access_token)
                        .await
                        .map_err(|error| handle_authenticated_error("Get model catalog", error))?,
                ),
            };
            let catalog_provider = catalog.as_ref().and_then(|catalog| {
                catalog
                    .providers
                    .iter()
                    .find(|candidate| candidate.id == provider.provider_key)
            });
            Some(model_provider_credentials(
                options.credentials_stdin,
                catalog_provider,
                settings,
            )?)
        }
        false => None,
    };
    let request = UpdateModelProviderRequest {
        display_name: options.name,
        auth_type: options.auth.map(Into::into),
        credentials,
    };
    let updated = context
        .client
        .update_model_provider(team.id, options.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Update model provider", error))?;
    writeln!(
        app.stdout,
        "Updated model provider {} ({}) for team {}.",
        escape_terminal(&updated.display_name),
        updated.id,
        team.id
    )?;
    Ok(())
}

pub(super) async fn remove_with(
    id: Uuid,
    team_override: Option<Uuid>,
    app: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, team_override).await?;
    context
        .client
        .delete_model_provider(team.id, id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Remove model provider", error))?;
    writeln!(
        app.stdout,
        "Removed model provider {id} from team {}.",
        team.id
    )?;
    Ok(())
}

impl From<DirectModelProviderAuth> for ModelProviderAuthType {
    fn from(value: DirectModelProviderAuth) -> Self {
        match value {
            DirectModelProviderAuth::ApiKey => Self::ApiKey,
            DirectModelProviderAuth::None => Self::None,
        }
    }
}
