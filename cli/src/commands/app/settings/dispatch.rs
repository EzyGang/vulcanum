use crate::commands::app::args::{
    GithubCommand, ModelProvidersCommand, ModelSelectionCommand, SettingsCommand,
    SettingsModelsCommand, SettingsTeamCommand, TaskTrackersCommand,
};
use crate::commands::app::settings::{
    self, device_oauth, github, model_providers, models, task_trackers,
};

pub(crate) async fn run(cmd: SettingsCommand) -> anyhow::Result<()> {
    match cmd {
        SettingsCommand::List { team } => settings::list(team).await,
        SettingsCommand::Team { cmd } => match cmd {
            SettingsTeamCommand::Set { team } => settings::set_team(team).await,
            SettingsTeamCommand::Clear => settings::clear_team().await,
        },
        SettingsCommand::Models { cmd } => match cmd {
            SettingsModelsCommand::Primary { cmd } => {
                run_model_selection(models::ModelSlot::Primary, cmd).await
            }
            SettingsModelsCommand::Small { cmd } => {
                run_model_selection(models::ModelSlot::Small, cmd).await
            }
        },
        SettingsCommand::TaskTrackers { cmd } => match cmd {
            TaskTrackersCommand::Add {
                name,
                instance_url,
                credentials_stdin,
                team,
            } => task_trackers::add(name, instance_url, credentials_stdin, team).await,
            TaskTrackersCommand::Update {
                id,
                name,
                instance_url,
                credentials_stdin,
                prompt_credentials,
                team,
            } => {
                task_trackers::update(task_trackers::UpdateOptions {
                    id,
                    name,
                    instance_url,
                    credentials_stdin,
                    prompt_credentials,
                    team,
                })
                .await
            }
            TaskTrackersCommand::Remove { id, team } => task_trackers::remove(id, team).await,
        },
        SettingsCommand::ModelProviders { cmd } => match cmd {
            ModelProvidersCommand::Add {
                provider_key,
                name,
                auth,
                credentials_stdin,
                team,
            } => {
                model_providers::add(model_providers::AddOptions {
                    provider_key,
                    name,
                    auth,
                    credentials_stdin,
                    team,
                })
                .await
            }
            ModelProvidersCommand::Update {
                id,
                name,
                auth,
                credentials_stdin,
                prompt_credentials,
                team,
            } => {
                model_providers::update(model_providers::UpdateOptions {
                    id,
                    name,
                    auth,
                    credentials_stdin,
                    prompt_credentials,
                    team,
                })
                .await
            }
            ModelProvidersCommand::Remove { id, team } => model_providers::remove(id, team).await,
            ModelProvidersCommand::ConnectOpenai {
                name,
                no_browser,
                team,
            } => device_oauth::connect_openai(name, no_browser, team).await,
        },
        SettingsCommand::Github { cmd } => match cmd {
            GithubCommand::Connect { no_browser, team } => github::connect(no_browser, team).await,
            GithubCommand::Disconnect { team } => github::disconnect(team).await,
        },
    }
}

async fn run_model_selection(
    slot: models::ModelSlot,
    cmd: ModelSelectionCommand,
) -> anyhow::Result<()> {
    match cmd {
        ModelSelectionCommand::Set {
            provider_key,
            model_id,
            team,
        } => models::set(slot, provider_key, model_id, team).await,
        ModelSelectionCommand::Clear { team } => models::clear(slot, team).await,
    }
}
