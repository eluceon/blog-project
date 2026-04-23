use async_trait::async_trait;

use crate::domain::{DomainError, Post};

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, title: &str, content: &str, author_id: i64)
    -> Result<Post, DomainError>;
    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError>;
    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, DomainError>;
    async fn delete(&self, id: i64) -> Result<(), DomainError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError>;
}
