mod client;
mod commands;
mod state;
mod token;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "vulcanum", about = "Vulcanum worker CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Worker commands (connect, daemon)
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
        /// Instance URL (e.g. http://localhost:8080)
        instance: String,
        /// Connection code from the instance
        #[arg(long)]
        code: String,
    },
    /// Run the worker daemon (poll loop, job execution)
    Daemon,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Worker { cmd } => match cmd {
            WorkerCommand::Connect { instance, code } => {
                commands::connect::run(code, instance).await
            }
            WorkerCommand::Daemon => commands::daemon::run().await,
        },
    }
}
