use serde::{Deserialize, Serialize};

pub const MAX_HIGHLIGHT_CHARS: usize = 8192;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCandidate {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSuggestion {
    pub source_highlight_id: String,
    pub bridging_sentence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentOutput {
    pub summary: String,
    pub tags: Vec<String>,
    #[serde(alias = "connection")]
    pub connection_suggestion: Option<ConnectionSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: String,
    pub text: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamHighlight {
    pub id: String,
    pub text: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedHighlight {
    pub id: String,
    pub text: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
    pub stream_id: String,
    pub stream_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListedHighlight {
    pub id: String,
    pub text: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub summary: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub connection_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub tier: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub pushed: usize,
    pub pulled: usize,
    pub conflicts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub record_type: String,
    pub record_id: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub resolved_at: Option<String>,
    pub resolution: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub logged_in: bool,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync: Option<String>,
    pub status: String,
    pub pending_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub loaded: bool,
    pub model_name: String,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorProfile {
    pub user_id: String,
    pub stripe_connect_account_id: Option<String>,
    pub is_verified: bool,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub platform_fee_percent: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetizedStream {
    pub stream_id: String,
    pub creator_id: String,
    pub monthly_price_cents: i64,
    pub subscriber_count: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorAnalytics {
    pub subscriber_count: i64,
    pub monthly_revenue_cents: i64,
    pub stream_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub subscriber_count: i64,
    pub is_trending: bool,
    pub weekly_views: i64,
}
