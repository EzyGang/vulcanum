mod commands;
mod console;
mod prompts;
#[cfg(test)]
mod tests;

#[cfg(not(target_os = "windows"))]
use anyhow::Context;
#[cfg(not(target_os = "windows"))]
use clap::ValueEnum;
use clap::{Parser, Subcommand};

use crate::commands::app::args::{
    ProjectReposCommand, ProjectsCommand, RunsCommand, SettingsCommand, WorkersCommand,
};
use crate::commands::app::board::args::BoardCommand;
use crate::commands::app::projects::args::{ProjectAutomationCommand, ProjectColumnsCommand};
#[cfg(not(target_os = "windows"))]
use crate::commands::setup::host::worker_server_path;
use crate::commands::skills::SkillsCommand;

#[derive(Parser)]
#[command(
    name = "vulcanum",
    about = "Vulcanum CLI",
    version = env!("VULCANUM_VERSION")
)]
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
    #[cfg(not(target_os = "windows"))]
    /// Manage the local worker lifecycle
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
    /// Inspect and add projects
    Projects {
        #[command(subcommand)]
        cmd: ProjectsCommand,
    },
    /// Browse and manage a configured project's task board
    Board {
        #[command(subcommand)]
        cmd: BoardCommand,
    },
    /// Inspect work runs
    Runs {
        #[command(subcommand)]
        cmd: RunsCommand,
    },
    /// Inspect and manage app settings
    Settings {
        #[command(subcommand)]
        cmd: SettingsCommand,
    },
    /// Install or print Vulcanum agent skills
    Skills {
        #[command(subcommand)]
        cmd: SkillsCommand,
    },
}

#[cfg(not(target_os = "windows"))]
#[derive(Subcommand)]
enum WorkerCommand {
    /// Run the worker daemon (poll loop, job execution)
    Daemon,
    /// Unregister this worker and remove local state
    #[command(name = "self-delete")]
    SelfDelete,
    /// Manage verified automatic worker-side updates
    Updates {
        #[command(subcommand)]
        cmd: WorkerUpdatesCommand,
    },
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
    },
}

#[cfg(not(target_os = "windows"))]
#[derive(Subcommand)]
enum WorkerUpdatesCommand {
    /// Enable verified automatic updates
    Enable,
    /// Disable verified automatic updates
    Disable,
}

#[cfg(not(target_os = "windows"))]
#[derive(Clone, Copy, Debug, ValueEnum)]
enum IsolationBackend {
    Kata,
    Docker,
    None,
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
        #[cfg(not(target_os = "windows"))]
        Command::Worker { cmd } => match cmd {
            WorkerCommand::Daemon => run_daemon_subcommand().await,
            WorkerCommand::SelfDelete => commands::self_delete::run().await,
            WorkerCommand::Updates { cmd } => match cmd {
                WorkerUpdatesCommand::Enable => commands::worker_updates::run(true),
                WorkerUpdatesCommand::Disable => commands::worker_updates::run(false),
            },
            WorkerCommand::Setup {
                instance,
                code,
                force,
                isolation,
            } => commands::setup::run(code, instance, force, isolation).await,
        },
        Command::Workers { cmd } => match cmd {
            WorkersCommand::List { team } => commands::app::workers::list(team).await,
            WorkersCommand::Rename {
                worker_id,
                name,
                team,
            } => commands::app::workers::rename(worker_id, &name, team).await,
        },
        Command::Projects { cmd } => match cmd {
            ProjectsCommand::List { team } => commands::app::projects::list(team).await,
            ProjectsCommand::Add {
                provider,
                workspace,
                project,
                repos,
                team,
            } => {
                commands::app::projects::add(commands::app::projects::AddOptions {
                    provider,
                    workspace,
                    project,
                    repos,
                    team,
                })
                .await
            }
            ProjectsCommand::Automation { cmd } => match cmd {
                ProjectAutomationCommand::Enable { project_id, team } => {
                    commands::app::projects::configuration::set_automation(project_id, true, team)
                        .await
                }
                ProjectAutomationCommand::Disable { project_id, team } => {
                    commands::app::projects::configuration::set_automation(project_id, false, team)
                        .await
                }
            },
            ProjectsCommand::Columns { cmd } => match cmd {
                ProjectColumnsCommand::Set {
                    project_id,
                    pickup,
                    in_progress,
                    in_review,
                    done,
                    team,
                } => {
                    commands::app::projects::configuration::set_columns(
                        commands::app::projects::configuration::ColumnsOptions {
                            project_id,
                            pickup,
                            in_progress,
                            in_review,
                            done,
                            team,
                        },
                    )
                    .await
                }
            },
            ProjectsCommand::Repos { cmd } => match cmd {
                ProjectReposCommand::List { team } => {
                    commands::app::projects::repos::list(team).await
                }
                ProjectReposCommand::Set {
                    project_id,
                    repos,
                    clear,
                    team,
                } => {
                    commands::app::projects::repos::set(
                        commands::app::projects::repos::EditOptions {
                            project_id,
                            repos,
                            clear,
                            team,
                        },
                    )
                    .await
                }
            },
        },
        Command::Board { cmd } => commands::app::board::run(cmd).await,
        Command::Runs { cmd } => match cmd {
            RunsCommand::List { team } => commands::app::runs::list(team).await,
        },
        Command::Settings { cmd } => commands::app::settings::dispatch::run(cmd).await,
        Command::Skills { cmd } => commands::skills::run(cmd).await,
    }
}

#[cfg(not(target_os = "windows"))]
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
