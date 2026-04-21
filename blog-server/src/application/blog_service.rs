use std::sync::Arc;

use crate::{
    data::post_repository::PostRepository,
    domain::{CreatePostRequest, DomainError, Post, UpdatePostRequest},
};

/// Application service for blog post CRUD operations.
pub struct BlogService {
    post_repo: Arc<dyn PostRepository>,
}

impl BlogService {
    /// Create a `BlogService` backed by the given repository.
    pub fn new(post_repo: Arc<dyn PostRepository>) -> Self {
        Self { post_repo }
    }

    /// Create a new post authored by `author_id`.
    pub async fn create_post(
        &self,
        req: &CreatePostRequest,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        self.post_repo
            .create(&req.title, &req.content, author_id)
            .await
    }

    /// Retrieve a post by its ID.
    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repo.find_by_id(id).await
    }

    /// Update a post, enforcing that `user_id` is the author.
    pub async fn update_post(
        &self,
        id: i64,
        req: &UpdatePostRequest,
        user_id: i64,
    ) -> Result<Post, DomainError> {
        let post = self.post_repo.find_by_id(id).await?;
        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }
        self.post_repo
            .update(id, req.title.as_deref(), req.content.as_deref())
            .await
    }

    /// Delete a post, enforcing that `user_id` is the author.
    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), DomainError> {
        let post = self.post_repo.find_by_id(id).await?;
        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }
        self.post_repo.delete(id).await
    }

    /// List posts with pagination, returning posts and total count.
    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), DomainError> {
        self.post_repo.list(limit, offset).await
    }
}
