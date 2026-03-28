#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Identity / Accounts ---

#[derive(Debug, Deserialize)]
pub struct IdentityResponse {
    pub accounts: Vec<AccountWithUser>,
}

#[derive(Debug, Deserialize)]
pub struct AccountWithUser {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub user: User,
}

#[derive(Debug, Deserialize)]
pub struct AccountSettings {
    pub id: String,
    pub name: String,
    pub cards_count: u64,
    pub created_at: DateTime<Utc>,
    pub auto_postpone_period_in_days: Option<u32>,
}

// --- Users ---

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: String,
    pub active: bool,
    pub email_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub avatar_url: Option<String>,
}

// --- Boards ---

#[derive(Debug, Deserialize, Clone)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub all_access: bool,
    pub created_at: DateTime<Utc>,
    pub auto_postpone_period_in_days: Option<u32>,
    pub url: String,
    pub creator: User,
    pub public_url: Option<String>,
}

// --- Columns ---

// Column color can be a plain string (column list) or an object (card show).
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ColumnColor {
    Plain(String),
    Structured { name: String, value: String },
}

impl std::fmt::Display for ColumnColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnColor::Plain(s) => write!(f, "{s}"),
            ColumnColor::Structured { name, .. } => write!(f, "{name}"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub color: ColumnColor,
    pub created_at: DateTime<Utc>,
}

// --- Tags ---

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub url: String,
}

// --- Cards ---

#[derive(Debug, Deserialize)]
pub struct Card {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub status: String,
    pub description: String,
    pub description_html: String,
    pub image_url: Option<String>,
    pub has_attachments: bool,
    pub tags: Vec<String>,
    // Only on show endpoint
    pub closed: Option<bool>,
    pub postponed: Option<bool>,
    pub golden: bool,
    pub last_active_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub board: Board,
    pub column: Option<Column>,
    pub creator: User,
    pub assignees: Option<Vec<User>>,
    pub has_more_assignees: Option<bool>,
    pub comments_url: Option<String>,
    pub reactions_url: Option<String>,
    pub steps: Option<Vec<Step>>,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    pub id: String,
    pub content: String,
    pub completed: bool,
}

// --- Comments ---

#[derive(Debug, Deserialize)]
pub struct CommentBody {
    pub plain_text: String,
    pub html: String,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body: CommentBody,
    pub creator: User,
    pub card: CommentCard,
    pub reactions_url: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct CommentCard {
    pub id: String,
    pub url: String,
}

// --- Reactions ---

#[derive(Debug, Deserialize)]
pub struct Reaction {
    pub id: String,
    pub content: String,
    pub reacter: User,
    pub url: String,
}

// --- Notifications ---

#[derive(Debug, Deserialize)]
pub struct Notification {
    pub id: String,
    pub read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub body: String,
    pub creator: User,
    pub card: NotificationCard,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct NotificationCard {
    pub id: String,
    pub title: String,
    pub status: String,
    pub url: String,
}

// --- Webhooks ---

#[derive(Debug, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub payload_url: String,
    pub active: bool,
    pub signing_secret: String,
    pub subscribed_actions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub board: Board,
}

// --- Auth ---

#[derive(Debug, Deserialize)]
pub struct PendingAuthResponse {
    pub pending_authentication_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionResponse {
    pub session_token: String,
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub token: String,
    pub description: String,
    pub permission: String,
}

// --- Request bodies ---

#[derive(Debug, Serialize)]
pub struct CreateBoardRequest {
    pub board: CreateBoardBody,
}

#[derive(Debug, Serialize)]
pub struct CreateBoardBody {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_postpone_period_in_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct UpdateBoardRequest {
    pub board: UpdateBoardBody,
}

#[derive(Debug, Serialize)]
pub struct UpdateBoardBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_postpone_period_in_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CreateCardRequest {
    pub card: CreateCardBody,
}

#[derive(Debug, Serialize)]
pub struct CreateCardBody {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct UpdateCardRequest {
    pub card: UpdateCardBody,
}

#[derive(Debug, Serialize)]
pub struct UpdateCardBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TriageRequest {
    pub column_id: String,
}

#[derive(Debug, Serialize)]
pub struct TaggingRequest {
    pub tag_title: String,
}

#[derive(Debug, Serialize)]
pub struct AssignmentRequest {
    pub assignee_id: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentRequest {
    pub comment: CreateCommentBody,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentBody {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateCommentRequest {
    pub comment: UpdateCommentBody,
}

#[derive(Debug, Serialize)]
pub struct UpdateCommentBody {
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct CreateReactionRequest {
    pub reaction: CreateReactionBody,
}

#[derive(Debug, Serialize)]
pub struct CreateReactionBody {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct CreateStepRequest {
    pub step: CreateStepBody,
}

#[derive(Debug, Serialize)]
pub struct CreateStepBody {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateStepRequest {
    pub step: UpdateStepBody,
}

#[derive(Debug, Serialize)]
pub struct UpdateStepBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CreateColumnRequest {
    pub column: CreateColumnBody,
}

#[derive(Debug, Serialize)]
pub struct CreateColumnBody {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateColumnRequest {
    pub column: UpdateColumnBody,
}

#[derive(Debug, Serialize)]
pub struct UpdateColumnBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateWebhookRequest {
    pub webhook: CreateWebhookBody,
}

#[derive(Debug, Serialize)]
pub struct CreateWebhookBody {
    pub name: String,
    pub url: String,
    pub subscribed_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateWebhookRequest {
    pub webhook: UpdateWebhookBody,
}

#[derive(Debug, Serialize)]
pub struct UpdateWebhookBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_actions: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct MagicLinkRequest {
    pub email_address: String,
}

#[derive(Debug, Serialize)]
pub struct MagicLinkCodeRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAccessTokenRequest {
    pub access_token: CreateAccessTokenBody,
}

#[derive(Debug, Serialize)]
pub struct CreateAccessTokenBody {
    pub description: String,
    pub permission: String,
}
