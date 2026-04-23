use std::sync::Arc;

use crate::{
    data::post_repository::PostRepository,
    domain::{CreatePostRequest, DomainError, Post, UpdatePostRequest},
};

pub struct BlogService {
    post_repo: Arc<dyn PostRepository>,
}

impl BlogService {
    pub fn new(post_repo: Arc<dyn PostRepository>) -> Self {
        Self { post_repo }
    }

    pub async fn create_post(
        &self,
        req: &CreatePostRequest,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        self.post_repo
            .create(&req.title, &req.content, author_id)
            .await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repo.find_by_id(id).await
    }

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

    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), DomainError> {
        let post = self.post_repo.find_by_id(id).await?;
        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }
        self.post_repo.delete(id).await
    }

    pub async fn list_posts(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Post>, i64), DomainError> {
        self.post_repo.list(limit, offset).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{CreatePostRequest, DomainError, UpdatePostRequest},
        test_mocks::{MockPostRepository, make_post},
    };

    #[tokio::test]
    async fn create_post_stores_and_returns_post() {
        let svc = BlogService::new(MockPostRepository::empty());
        let req = CreatePostRequest {
            title: "Hello".to_owned(),
            content: "World".to_owned(),
        };
        let post = svc.create_post(&req, 7).await.unwrap();
        assert_eq!(post.title, "Hello");
        assert_eq!(post.content, "World");
        assert_eq!(post.author_id, 7);
    }

    #[tokio::test]
    async fn get_post_not_found_returns_error() {
        let svc = BlogService::new(MockPostRepository::empty());
        assert!(matches!(
            svc.get_post(99).await,
            Err(DomainError::PostNotFound)
        ));
    }

    #[tokio::test]
    async fn update_post_by_author_succeeds() {
        let repo = MockPostRepository::with(vec![make_post(1, 10)]);
        let svc = BlogService::new(repo);
        let req = UpdatePostRequest {
            title: Some("Updated".to_owned()),
            content: None,
        };
        let post = svc.update_post(1, &req, 10).await.unwrap();
        assert_eq!(post.title, "Updated");
        assert_eq!(post.content, "Content");
    }

    #[tokio::test]
    async fn update_post_by_non_author_is_forbidden() {
        let repo = MockPostRepository::with(vec![make_post(1, 10)]);
        let svc = BlogService::new(repo);
        let req = UpdatePostRequest {
            title: Some("Hijacked".to_owned()),
            content: None,
        };
        assert!(matches!(
            svc.update_post(1, &req, 99).await,
            Err(DomainError::Forbidden)
        ));
    }

    #[tokio::test]
    async fn delete_post_by_author_succeeds() {
        let repo = MockPostRepository::with(vec![make_post(1, 10)]);
        let svc = BlogService::new(repo);
        assert!(svc.delete_post(1, 10).await.is_ok());
    }

    #[tokio::test]
    async fn delete_post_by_non_author_is_forbidden() {
        let repo = MockPostRepository::with(vec![make_post(1, 10)]);
        let svc = BlogService::new(repo);
        assert!(matches!(
            svc.delete_post(1, 99).await,
            Err(DomainError::Forbidden)
        ));
    }

    #[tokio::test]
    async fn list_posts_returns_correct_page_and_total() {
        let posts = (1..=5).map(|i| make_post(i, 1)).collect();
        let svc = BlogService::new(MockPostRepository::with(posts));
        let (page, total) = svc.list_posts(3, 1).await.unwrap();
        assert_eq!(total, 5);
        assert_eq!(page.len(), 3);
    }
}
