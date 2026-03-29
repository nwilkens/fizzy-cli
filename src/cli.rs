use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fizzyctl", about = "Fizzy project management CLI", version)]
pub struct Cli {
    /// Output raw JSON instead of formatted text
    #[arg(long, global = true)]
    pub json: bool,

    /// Account slug to use (overrides config default)
    #[arg(long, short = 'a', global = true)]
    pub account: Option<String>,

    /// Base URL of the Fizzy instance
    #[arg(long, global = true)]
    pub url: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Fizzy
    Login {
        /// Paste a personal access token directly
        #[arg(long)]
        token: Option<String>,
    },

    /// Log out and remove stored credentials
    Logout,

    /// List your accounts
    Accounts,

    /// Show current configuration
    Config,

    /// Set a configuration value
    Set {
        /// Key to set (account, url)
        key: String,
        /// Value
        value: String,
    },

    /// List boards in the current account
    Boards,

    /// Work with a specific board
    #[command(subcommand)]
    Board(BoardCommand),

    /// List cards with filters
    Cards {
        /// Filter by board name or ID
        #[arg(long, short = 'b')]
        board: Option<String>,
        /// Filter by column name or ID
        #[arg(long)]
        column: Option<String>,
        /// Filter by assignee name or ID
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Filter: all, closed, not_now, stalled, postponing_soon, golden
        #[arg(long)]
        index: Option<String>,
        /// Sort: latest, newest, oldest
        #[arg(long)]
        sort: Option<String>,
        /// Search terms
        #[arg(long)]
        search: Option<String>,
        /// Fetch all pages
        #[arg(long)]
        all: bool,
    },

    /// Work with a specific card
    #[command(subcommand)]
    Card(CardCommand),

    /// List columns for a board
    Columns {
        /// Board ID or name
        board: String,
    },

    /// Work with a specific column
    #[command(subcommand)]
    Column(ColumnCommand),

    /// List users in the current account
    Users,

    /// Show a specific user
    User {
        /// User ID
        id: String,
    },

    /// List tags in the current account
    Tags,

    // --- Agent workflow commands ---

    /// Initialize a project for fizzyctl (board, columns, hooks, CLAUDE.md)
    Init {
        /// Board name (defaults to <dirname>-<hash>)
        #[arg(long)]
        name: Option<String>,
        /// Use an existing board by ID or name instead of creating one
        #[arg(long, short = 'b')]
        board: Option<String>,
    },

    /// Show current identity and user info
    Whoami,

    /// Inject task context for AI agents (compact, token-efficient)
    Prime {
        /// Board name or ID (uses default board from config if set)
        #[arg(long, short = 'b')]
        board: Option<String>,
    },

    /// Show cards ready for pickup (dependency-aware)
    Ready {
        /// Board name or ID
        #[arg(long, short = 'b')]
        board: Option<String>,
    },

    /// Show cards blocked by unsatisfied dependencies
    Blocked {
        /// Board name or ID
        #[arg(long, short = 'b')]
        board: Option<String>,
    },

    /// Add a dependency: card depends on another (#after-N tag)
    Dep {
        /// Card number that has the dependency
        number: u64,
        /// Card number it depends on
        depends_on: u64,
    },

    /// View or set the plan on a card (💡 comment)
    Plan {
        /// Card number
        number: u64,
        /// Plan text (if omitted, shows existing plan)
        text: Option<String>,
    },

    /// Claim a card: assign to self and move to "In Progress"
    Claim {
        /// Card number
        number: u64,
    },

    /// Add a progress comment to a card
    Progress {
        /// Card number
        number: u64,
        /// Progress message
        message: String,
    },

    /// Mark card done: close and optionally add a final comment
    Done {
        /// Card number
        number: u64,
        /// Optional closing comment
        message: Option<String>,
    },

    /// Move card to "Review" column for human review
    Review {
        /// Card number
        number: u64,
        /// Optional review comment
        message: Option<String>,
    },

    /// List pinned cards
    Pins,

    /// List notifications
    Notifications {
        /// Mark all as read
        #[arg(long)]
        read_all: bool,
    },

    /// Mark a notification as read
    #[command(name = "notification-read")]
    NotificationRead {
        /// Notification ID
        id: String,
    },

    /// Mark a notification as unread
    #[command(name = "notification-unread")]
    NotificationUnread {
        /// Notification ID
        id: String,
    },

    /// List webhooks for a board
    Webhooks {
        /// Board ID or name
        board: String,
    },

    /// Work with a specific webhook
    #[command(subcommand)]
    Webhook(WebhookCommand),
}

#[derive(Subcommand)]
pub enum BoardCommand {
    /// Show a board
    Show {
        /// Board ID or name
        id: String,
    },
    /// Create a board
    Create {
        /// Board name
        name: String,
        /// Make accessible to all users
        #[arg(long)]
        all_access: Option<bool>,
        /// Auto-postpone period in days
        #[arg(long)]
        entropy: Option<u32>,
    },
    /// Update a board
    Update {
        /// Board ID or name
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// All access
        #[arg(long)]
        all_access: Option<bool>,
        /// Auto-postpone period in days
        #[arg(long)]
        entropy: Option<u32>,
    },
    /// Delete a board
    Delete {
        /// Board ID or name
        id: String,
    },
    /// Publish a board
    Publish {
        /// Board ID or name
        id: String,
    },
    /// Unpublish a board
    Unpublish {
        /// Board ID or name
        id: String,
    },
}

#[derive(Subcommand)]
pub enum CardCommand {
    /// Show card details
    Show {
        /// Card number
        number: u64,
    },
    /// Create a new card
    Create {
        /// Card title
        title: String,
        /// Board name or ID
        #[arg(long, short = 'b')]
        board: String,
        /// Card description
        #[arg(long, short = 'd')]
        description: Option<String>,
        /// Tags (comma-separated titles)
        #[arg(long)]
        tags: Option<String>,
        /// Create as draft
        #[arg(long)]
        draft: bool,
    },
    /// Update a card
    Update {
        /// Card number
        number: u64,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(long, short = 'd')]
        description: Option<String>,
    },
    /// Delete a card
    Delete {
        /// Card number
        number: u64,
    },
    /// Close a card
    Close {
        /// Card number
        number: u64,
    },
    /// Reopen a closed card
    Reopen {
        /// Card number
        number: u64,
    },
    /// Postpone a card (move to Not Now)
    Postpone {
        /// Card number
        number: u64,
    },
    /// Triage a card into a column
    Triage {
        /// Card number
        number: u64,
        /// Column name or ID
        #[arg(long, short = 'c')]
        column: String,
    },
    /// Send a card back to triage
    Untriage {
        /// Card number
        number: u64,
    },
    /// Toggle a tag on a card
    Tag {
        /// Card number
        number: u64,
        /// Tag title
        tag: String,
    },
    /// Toggle user assignment on a card
    Assign {
        /// Card number
        number: u64,
        /// User name or ID
        user: String,
    },
    /// Watch a card
    Watch {
        /// Card number
        number: u64,
    },
    /// Unwatch a card
    Unwatch {
        /// Card number
        number: u64,
    },
    /// Toggle golden status
    Gold {
        /// Card number
        number: u64,
    },
    /// Remove golden status
    Ungold {
        /// Card number
        number: u64,
    },
    /// Pin a card
    Pin {
        /// Card number
        number: u64,
    },
    /// Unpin a card
    Unpin {
        /// Card number
        number: u64,
    },
    /// List or add comments on a card
    Comment {
        /// Card number
        number: u64,
        /// Comment body (if omitted, lists comments)
        body: Option<String>,
    },
    /// Update a comment
    #[command(name = "comment-update")]
    CommentUpdate {
        /// Card number
        number: u64,
        /// Comment ID
        comment_id: String,
        /// New body
        body: String,
    },
    /// Delete a comment
    #[command(name = "comment-delete")]
    CommentDelete {
        /// Card number
        number: u64,
        /// Comment ID
        comment_id: String,
    },
    /// Add a reaction to a card
    React {
        /// Card number
        number: u64,
        /// Reaction content (emoji, max 16 chars)
        content: String,
    },
    /// Remove a reaction from a card
    Unreact {
        /// Card number
        number: u64,
        /// Reaction ID
        reaction_id: String,
    },
    /// List reactions on a card
    Reactions {
        /// Card number
        number: u64,
    },
    /// Add a step to a card
    #[command(name = "step-add")]
    StepAdd {
        /// Card number
        number: u64,
        /// Step content
        content: String,
    },
    /// Complete a step
    #[command(name = "step-complete")]
    StepComplete {
        /// Card number
        number: u64,
        /// Step ID
        step_id: String,
    },
    /// Delete a step
    #[command(name = "step-delete")]
    StepDelete {
        /// Card number
        number: u64,
        /// Step ID
        step_id: String,
    },
    /// Add a reaction to a comment
    #[command(name = "comment-react")]
    CommentReact {
        /// Card number
        number: u64,
        /// Comment ID
        comment_id: String,
        /// Reaction content
        content: String,
    },
    /// Remove a reaction from a comment
    #[command(name = "comment-unreact")]
    CommentUnreact {
        /// Card number
        number: u64,
        /// Comment ID
        comment_id: String,
        /// Reaction ID
        reaction_id: String,
    },
    /// List reactions on a comment
    #[command(name = "comment-reactions")]
    CommentReactions {
        /// Card number
        number: u64,
        /// Comment ID
        comment_id: String,
    },
}

#[derive(Subcommand)]
pub enum ColumnCommand {
    /// Create a column
    Create {
        /// Board ID or name
        board: String,
        /// Column name
        name: String,
        /// Color (Blue, Gray, Tan, Yellow, Lime, Aqua, Violet, Purple, Pink)
        #[arg(long)]
        color: Option<String>,
    },
    /// Update a column
    Update {
        /// Board ID or name
        board: String,
        /// Column ID
        column_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New color
        #[arg(long)]
        color: Option<String>,
    },
    /// Delete a column
    Delete {
        /// Board ID or name
        board: String,
        /// Column ID
        column_id: String,
    },
}

#[derive(Subcommand)]
pub enum WebhookCommand {
    /// Show a webhook
    Show {
        /// Board ID or name
        board: String,
        /// Webhook ID
        id: String,
    },
    /// Create a webhook
    Create {
        /// Board ID or name
        board: String,
        /// Webhook name
        #[arg(long)]
        name: String,
        /// Payload URL
        #[arg(long = "payload-url")]
        payload_url: String,
        /// Subscribed actions (comma-separated)
        #[arg(long)]
        actions: String,
    },
    /// Update a webhook
    Update {
        /// Board ID or name
        board: String,
        /// Webhook ID
        id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New subscribed actions (comma-separated)
        #[arg(long)]
        actions: Option<String>,
    },
    /// Delete a webhook
    Delete {
        /// Board ID or name
        board: String,
        /// Webhook ID
        id: String,
    },
    /// Activate a webhook
    Activate {
        /// Board ID or name
        board: String,
        /// Webhook ID
        id: String,
    },
}
