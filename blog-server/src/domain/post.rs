use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A blog post stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Unique post identifier.
    pub id: i64,
    /// Post title.
    pub title: String,
    /// Full post content.
    pub content: String,
    /// ID of the user who created the post.
    pub author_id: i64,
    /// Username of the author, joined from the users table.
    pub author_username: String,
    /// When the post was first published.
    pub created_at: DateTime<Utc>,
    /// When the post was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Request body for creating a new post.
#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    /// Title of the new post.
    pub title: String,
    /// Body text of the new post.
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
