mod commands;
mod console;

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};

use crate::commands::setup::utils::worker_server_path;

#[derive(Parser)]
#[command(name = "vulcanum", about = "Vulcanum worker CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Worker commands (daemon, setup)
    #[command(visible_alias = "wrk")]
    Worker {
        #[command(subcommand)]
        cmd: WorkerCommand,
    },
}

#[derive(Subcommand)]
enum WorkerCommand {
    /// Run the worker daemon (poll loop, job execution)
    Daemon,
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
        /// Isolation backend to use (kata, gvisor, or none)
        #[arg(long, value_enum)]
        isolation: Option<IsolationBackend>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum IsolationBackend {
    Kata,
    Gvisor,
    None,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vulcanum_shared::telemetry::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Worker { cmd } => match cmd {
            WorkerCommand::Daemon => run_daemon_subcommand().await,
            WorkerCommand::Setup {
                instance,
                code,
                force,
                isolation,
            } => commands::setup::run(code, instance, force, isolation).await,
        },
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
