use clap::{Subcommand, ValueEnum};
use uuid::Uuid;

#[derive(Subcommand)]
pub(crate) enum WorkersCommand {
    /// List workers for a team
    List {
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Subcommand)]
pub(crate) enum RunsCommand {
    /// List work runs for a team
    List {
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Subcommand)]
pub(crate) enum SettingsCommand {
    /// List settings for a team
    List {
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Manage the local team pin
    Team {
        #[command(subcommand)]
        cmd: SettingsTeamCommand,
    },
    /// Manage primary and small model selection
    Models {
        #[command(subcommand)]
        cmd: SettingsModelsCommand,
    },
    /// Manage task tracker connections
    TaskTrackers {
        #[command(subcommand)]
        cmd: TaskTrackersCommand,
    },
    /// Manage model provider connections
    ModelProviders {
        #[command(subcommand)]
        cmd: ModelProvidersCommand,
    },
    /// Manage the GitHub App connection
    Github {
        #[command(subcommand)]
        cmd: GithubCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum SettingsTeamCommand {
    /// Pin a team for app-facing commands
    Set { team: Uuid },
    /// Clear or reset the local team pin
    Clear,
}

#[derive(Subcommand)]
pub(crate) enum SettingsModelsCommand {
    /// Manage the primary model
    Primary {
        #[command(subcommand)]
        cmd: ModelSelectionCommand,
    },
    /// Manage the small model
    Small {
        #[command(subcommand)]
        cmd: ModelSelectionCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum ModelSelectionCommand {
    /// Select a connected provider and catalog model
    Set {
        provider_key: String,
        model_id: String,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Clear the selected provider and model
    Clear {
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Subcommand)]
pub(crate) enum TaskTrackersCommand {
    /// Add a task tracker
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        instance_url: String,
        /// Read credentials as JSON from stdin
        #[arg(long)]
        credentials_stdin: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Update a task tracker
    Update {
        id: Uuid,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        instance_url: Option<String>,
        /// Read replacement credentials as JSON from stdin
        #[arg(long, conflicts_with = "prompt_credentials")]
        credentials_stdin: bool,
        /// Prompt for replacement credentials with input hidden
        #[arg(long, conflicts_with = "credentials_stdin")]
        prompt_credentials: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Remove a task tracker
    Remove {
        id: Uuid,
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum DirectModelProviderAuth {
    ApiKey,
    None,
}

#[derive(Subcommand)]
pub(crate) enum ModelProvidersCommand {
    /// Add a model provider
    Add {
        provider_key: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, value_enum, default_value = "api-key")]
        auth: DirectModelProviderAuth,
        /// Read credentials as JSON from stdin
        #[arg(long)]
        credentials_stdin: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Update a model provider
    Update {
        id: Uuid,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, value_enum)]
        auth: Option<DirectModelProviderAuth>,
        /// Read replacement credentials as JSON from stdin
        #[arg(long, conflicts_with = "prompt_credentials")]
        credentials_stdin: bool,
        /// Prompt for replacement credentials with input hidden
        #[arg(long, conflicts_with = "credentials_stdin")]
        prompt_credentials: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Remove a model provider
    Remove {
        id: Uuid,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Connect OpenAI with device OAuth
    ConnectOpenai {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        no_browser: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Subcommand)]
pub(crate) enum GithubCommand {
    /// Start GitHub App installation
    Connect {
        #[arg(long)]
        no_browser: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Disconnect the installed GitHub App
    Disconnect {
        #[arg(long)]
        team: Option<Uuid>,
    },
}
