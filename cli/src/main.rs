mod commands;
mod console;
mod prompts;
#[cfg(test)]
mod tests;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use uuid::Uuid;

use crate::commands::app::args::{
    GithubCommand, ModelProvidersCommand, ModelSelectionCommand, SettingsCommand,
    SettingsModelsCommand, SettingsTeamCommand, TaskTrackersCommand,
};

use crate::commands::setup::host::worker_server_path;

#[derive(Parser)]
#[command(name = "vulcanum", about = "Vulcanum CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Log in for app-facing commands
    Login {
        /// Instance URL (e.g. https://vulcanum.example.com)
        #[arg(long)]
        instance: Option<String>,
        /// Read the single-user instance password from stdin
        #[arg(long)]
        password_stdin: bool,
        /// Exchange an existing multi-user one-time code
        #[arg(long)]
        auth_code: Option<String>,
        /// Print the multi-user login URL without opening a browser
        #[arg(long)]
        no_browser: bool,
    },
    /// Worker commands (daemon, setup)
    #[command(visible_alias = "wrk")]
    Worker {
        #[command(subcommand)]
        cmd: WorkerCommand,
    },
    /// Inspect registered workers
    Workers {
        #[command(subcommand)]
        cmd: WorkersCommand,
    },
    /// Inspect and manage app settings
    Settings {
        #[command(subcommand)]
        cmd: SettingsCommand,
    },
}

#[derive(Subcommand)]
enum WorkerCommand {
    /// Run the worker daemon (poll loop, job execution)
    Daemon,
    /// Unregister this worker and remove local state
    #[command(name = "self-delete")]
    SelfDelete,
    /// Install dependencies, configure systemd, and register with an instance
    Setup {
        /// Instance URL (e.g. https://vulcanum.example.com)
        #[arg(long)]
        instance: Option<String>,
        /// Connection code from the instance
        #[arg(long)]
        code: Option<String>,
        /// Force re-registration even if already connected
        #[arg(long)]
        force: bool,
        #[arg(
            long,
            value_enum,
            help = "Isolation backend. Defaults to docker when --instance and --code are supplied."
        )]
        isolation: Option<IsolationBackend>,
        /// Agent backend to use (opencode or omp-rpc)
        #[arg(long, value_enum)]
        agent_backend: Option<AgentBackendArg>,
    },
}

#[derive(Subcommand)]
enum WorkersCommand {
    /// List workers for a team
    List {
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum IsolationBackend {
    Kata,
    Docker,
    None,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum AgentBackendArg {
    Opencode,
    OmpRpc,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vulcanum_shared::telemetry::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Login {
            instance,
            password_stdin,
            auth_code,
            no_browser,
        } => commands::login::run(instance, password_stdin, auth_code, no_browser).await,
        Command::Worker { cmd } => match cmd {
            WorkerCommand::Daemon => run_daemon_subcommand().await,
            WorkerCommand::SelfDelete => commands::self_delete::run().await,
            WorkerCommand::Setup {
                instance,
                code,
                force,
                isolation,
                agent_backend,
            } => commands::setup::run(code, instance, force, isolation, agent_backend).await,
        },
        Command::Workers { cmd } => match cmd {
            WorkersCommand::List { team } => commands::app::workers::list(team).await,
        },
        Command::Settings { cmd } => run_settings_command(cmd).await,
    }
}

async fn run_settings_command(cmd: SettingsCommand) -> anyhow::Result<()> {
    match cmd {
        SettingsCommand::List { team } => commands::app::settings::list(team).await,
        SettingsCommand::Team { cmd } => match cmd {
            SettingsTeamCommand::Set { team } => commands::app::settings::set_team(team).await,
            SettingsTeamCommand::Clear => commands::app::settings::clear_team().await,
        },
        SettingsCommand::Models { cmd } => match cmd {
            SettingsModelsCommand::Primary { cmd } => {
                run_model_selection(commands::app::settings::models::ModelSlot::Primary, cmd).await
            }
            SettingsModelsCommand::Small { cmd } => {
                run_model_selection(commands::app::settings::models::ModelSlot::Small, cmd).await
            }
        },
        SettingsCommand::TaskTrackers { cmd } => match cmd {
            TaskTrackersCommand::Add {
                name,
                instance_url,
                credentials_stdin,
                team,
            } => {
                commands::app::settings::task_trackers::add(
                    name,
                    instance_url,
                    credentials_stdin,
                    team,
                )
                .await
            }
            TaskTrackersCommand::Update {
                id,
                name,
                instance_url,
                credentials_stdin,
                prompt_credentials,
                team,
            } => {
                commands::app::settings::task_trackers::update(
                    commands::app::settings::task_trackers::UpdateOptions {
                        id,
                        name,
                        instance_url,
                        credentials_stdin,
                        prompt_credentials,
                        team,
                    },
                )
                .await
            }
            TaskTrackersCommand::Remove { id, team } => {
                commands::app::settings::task_trackers::remove(id, team).await
            }
        },
        SettingsCommand::ModelProviders { cmd } => match cmd {
            ModelProvidersCommand::Add {
                provider_key,
                name,
                auth,
                credentials_stdin,
                team,
            } => {
                commands::app::settings::model_providers::add(
                    commands::app::settings::model_providers::AddOptions {
                        provider_key,
                        name,
                        auth,
                        credentials_stdin,
                        team,
                    },
                )
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
                commands::app::settings::model_providers::update(
                    commands::app::settings::model_providers::UpdateOptions {
                        id,
                        name,
                        auth,
                        credentials_stdin,
                        prompt_credentials,
                        team,
                    },
                )
                .await
            }
            ModelProvidersCommand::Remove { id, team } => {
                commands::app::settings::model_providers::remove(id, team).await
            }
            ModelProvidersCommand::ConnectOpenai {
                name,
                no_browser,
                team,
            } => {
                commands::app::settings::device_oauth::connect_openai(name, no_browser, team).await
            }
        },
        SettingsCommand::Github { cmd } => match cmd {
            GithubCommand::Connect { no_browser, team } => {
                commands::app::settings::github::connect(no_browser, team).await
            }
            GithubCommand::Disconnect { team } => {
                commands::app::settings::github::disconnect(team).await
            }
        },
    }
}

async fn run_model_selection(
    slot: commands::app::settings::models::ModelSlot,
    cmd: ModelSelectionCommand,
) -> anyhow::Result<()> {
    match cmd {
        ModelSelectionCommand::Set {
            provider_key,
            model_id,
            team,
        } => commands::app::settings::models::set(slot, provider_key, model_id, team).await,
        ModelSelectionCommand::Clear { team } => {
            commands::app::settings::models::clear(slot, team).await
        }
    }
}

async fn run_daemon_subcommand() -> anyhow::Result<()> {
    let path = worker_server_path()?;
    let mut child = tokio::process::Command::new(&path)
        .spawn()
        .with_context(|| format!("failed to spawn {path}"))?;
    let status = child
        .wait()
        .await
        .with_context(|| format!("failed to wait for {path}"))?;
    std::process::exit(status.code().unwrap_or(1));
}
