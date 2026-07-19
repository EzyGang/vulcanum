use clap::Subcommand;
use uuid::Uuid;

#[derive(Subcommand)]
pub(crate) enum BoardCommand {
    /// Show a compact task table grouped by board column
    View {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Maximum tasks shown per column
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// List one board column with pagination
    Column {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Column name or slug
        column: String,
        /// One-based page number
        #[arg(long, default_value_t = 1)]
        page: usize,
        /// Tasks per page
        #[arg(long, default_value_t = 20)]
        page_size: usize,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Create, inspect, edit, move, or search tasks
    Tasks {
        #[command(subcommand)]
        cmd: BoardTasksCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum BoardTasksCommand {
    /// Create a task in a configured project
    Create {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Task title
        title: String,
        /// Task body; use --body-stdin for multiline input
        #[arg(long, conflicts_with = "body_stdin")]
        body: Option<String>,
        /// Read the complete multiline task body from stdin
        #[arg(long)]
        body_stdin: bool,
        /// Initial provider status or column slug
        #[arg(long)]
        status: Option<String>,
        /// Initial provider priority
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Show one task selected by provider ID or task slug
    Get {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Provider task ID or task slug, such as VLC-42
        task: String,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Edit one task selected by provider ID or task slug
    Edit {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Provider task ID or task slug, such as VLC-42
        task: String,
        /// Replacement title; omitted values remain unchanged
        #[arg(long)]
        title: Option<String>,
        /// Replacement body; use --body-stdin for multiline input
        #[arg(long, conflicts_with = "body_stdin")]
        body: Option<String>,
        /// Read the complete replacement body from stdin
        #[arg(long)]
        body_stdin: bool,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Move one task to another column
    Move {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Provider task ID or task slug, such as VLC-42
        task: String,
        /// Destination column name or slug
        column: String,
        #[arg(long)]
        team: Option<Uuid>,
    },
    /// Search and filter tasks, showing task slug and title
    Search {
        /// Configured project UUID from `vulcanum projects list`
        project_id: Uuid,
        /// Case-insensitive text matched against slug, title, and body
        #[arg(long)]
        query: Option<String>,
        /// Only tasks in this column name or slug
        #[arg(long)]
        column: Option<String>,
        /// Only tasks carrying this label name or ID
        #[arg(long)]
        label: Option<String>,
        /// One-based page number
        #[arg(long, default_value_t = 1)]
        page: usize,
        /// Tasks per page
        #[arg(long, default_value_t = 20)]
        page_size: usize,
        #[arg(long)]
        team: Option<Uuid>,
    },
}
