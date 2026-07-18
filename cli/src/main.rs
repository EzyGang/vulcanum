mod commands;
mod console;
mod prompts;
#[cfg(test)]
mod tests;

use crate::commands::app::args::{
    ProjectReposCommand, ProjectsCommand, RunsCommand, SettingsCommand, WorkersCommand,
};
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};

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
    /// Inspect and add projects
    Projects {
        #[command(subcommand)]
        cmd: ProjectsCommand,
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
        Command::Runs { cmd } => match cmd {
            RunsCommand::List { team } => commands::app::runs::list(team).await,
        },
        Command::Settings { cmd } => commands::app::settings::dispatch::run(cmd).await,
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
