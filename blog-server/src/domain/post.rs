use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

/// Request body for updating an existing post.
///
/// Both fields are optional; `None` means "leave unchanged".
#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    /// New title, or `None` to keep the existing title.
    pub title: Option<String>,
    /// New content, or `None` to keep the existing content.
    pub content: Option<String>,
}
