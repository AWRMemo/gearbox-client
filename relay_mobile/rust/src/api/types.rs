#[derive(Debug, Clone)]
pub struct SearchResultResponse {
    pub id: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct ListedHighlightResponse {
    pub id: String,
    pub text: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub connection_suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StreamInfoResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct AuthStatusResponse {
    pub logged_in: bool,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthResultResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct SyncStatusResponse {
    pub status: String,
    pub pending_count: i64,
}

#[derive(Debug, Clone)]
pub struct SyncReportResponse {
    pub pushed: i64,
    pub pulled: i64,
    pub conflicts: i64,
}

#[derive(Debug, Clone)]
pub struct ConflictResponse {
    pub id: String,
    pub record_type: String,
    pub record_id: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub resolved_at: Option<String>,
    pub resolution: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct FeedHighlightResponse {
    pub id: String,
    pub text: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub stream_title: String,
}

#[derive(Debug, Clone)]
pub struct ModelStatusResponse {
    pub loaded: bool,
    pub model_name: String,
    pub backend: String,
}

#[derive(Debug, Clone)]
pub struct SourceMetaResponse {
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
}
