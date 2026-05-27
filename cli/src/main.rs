mod commands;
mod console;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vulcanum", about = "Vulcanum worker CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Worker commands (connect, daemon, setup)
    #[command(visible_alias = "wrk")]
    Worker {
        #[command(subcommand)]
        cmd: WorkerCommand,
    },
}

#[derive(Subcommand)]
enum WorkerCommand {
    /// Register a worker with an instance using a connection code
    Connect {
        /// Instance URL (e.g. https://vulcanum.example.com)
        instance: Option<String>,
        /// Connection code from the instance
        #[arg(long)]
        code: Option<String>,
    },
    /// Run the worker daemon (poll loop, job execution)
    Daemon,
    /// Install Docker, Kata Containers, pull agent image, configure systemd, and register with an instance
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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vulcanum_shared::telemetry::init();

    let cli = Cli::parse();

    match cli.command {
        Command::Worker { cmd } => match cmd {
            WorkerCommand::Connect { instance, code } => {
                commands::connect::run(code, instance).await
            }
            WorkerCommand::Daemon => run_daemon_subcommand().await,
            WorkerCommand::Setup {
                instance,
                code,
                force,
            } => commands::setup::run(code, instance, force).await,
        },
    }
}

async fn run_daemon_subcommand() -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("failed to get current exe")?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to get exe directory"))?;
    let name = if cfg!(windows) {
        "vulcanum-worker-server.exe"
    } else {
        "vulcanum-worker-server"
    };
    let path = dir.join(name);
    if !path.exists() {
        anyhow::bail!("worker-server binary not found at {}", path.display());
    }
    let mut child = tokio::process::Command::new(&path)
        .spawn()
        .with_context(|| format!("failed to spawn {}", path.display()))?;
    let status = child
        .wait()
        .await
        .with_context(|| format!("failed to wait for {}", path.display()))?;
    std::process::exit(status.code().unwrap_or(1));
}
