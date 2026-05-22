mod api_error;
mod client;
mod commands;
mod harness;
mod state;
mod token;

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
        /// Instance URL (e.g. http://localhost:8080)
        instance: String,
        /// Connection code from the instance
        #[arg(long)]
        code: String,
    },
    /// Run the worker daemon (poll loop, job execution)
    Daemon,
    /// Install Docker, Kata Containers, pull agent image, and configure systemd
    Setup,
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
            WorkerCommand::Daemon => commands::daemon::run().await,
            WorkerCommand::Setup => commands::setup::run().await,
        },
    }
}
