use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand)]
pub(crate) enum ProjectAutomationCommand {
    /// Enable automation for a configured project
    Enable {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Disable automation for a configured project
    Disable {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        #[arg(long)]
        team: Option<Uuid>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ProjectColumnsCommand {
    /// Mark one or more automation workflow columns
    Set {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Column name, slug, or provider ID used to pick up new tasks
        #[arg(long)]
        pickup: Option<String>,
        /// Column name, slug, or provider ID used while implementation is active
        #[arg(long)]
        in_progress: Option<String>,
        /// Column name, slug, or provider ID used while review is active
        #[arg(long)]
        in_review: Option<String>,
        /// Column name, slug, or provider ID used for completed tasks
        #[arg(long)]
        done: Option<String>,
        #[arg(long)]
        team: Option<Uuid>,
    },
}
