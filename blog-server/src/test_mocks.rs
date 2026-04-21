//! Shared in-memory fakes for `UserRepository` and `PostRepository`.
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;

use crate::{
    data::{post_repository::PostRepository, user_repository::UserRepository},
    domain::{DomainError, Post, RegisterUserRequest, User},
    infrastructure::jwt::JwtService,
};

pub const TEST_JWT_SECRET: &str = "test-jwt-secret-minimum-32-chars!!";

pub fn make_jwt() -> Arc<JwtService> {
    Arc::new(JwtService::new(TEST_JWT_SECRET))
}

/// Build a `Post` fixture with the given id and author_id.
pub fn make_post(id: i64, author_id: i64) -> Post {
    Post {
        id,
        title: format!("Post {id}"),
        content: "Content".to_owned(),
        author_id,
        author_username: "author".to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub struct MockPostRepository {
    posts: Mutex<Vec<Post>>,
}

impl MockPostRepository {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            posts: Mutex::new(vec![]),
        })
    }
    pub fn with(posts: Vec<Post>) -> Arc<Self> {
        Arc::new(Self {
            posts: Mutex::new(posts),
        })
    }
}

#[async_trait]
impl PostRepository for MockPostRepository {
    async fn create(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        let mut posts = self.posts.lock().unwrap();
        let id = posts.len() as i64 + 1;
        let post = Post {
            id,
            title: title.to_owned(),
            content: content.to_owned(),
            author_id,
            author_username: "author".to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        posts.push(post.clone());
        Ok(post)
    }

    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError> {
        self.posts
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or(DomainError::PostNotFound)
    }

    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, DomainError> {
        let mut posts = self.posts.lock().unwrap();
        let post = posts
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(DomainError::PostNotFound)?;
        if let Some(t) = title {
            post.title = t.to_owned();
        }
        if let Some(c) = content {
            post.content = c.to_owned();
        }
        Ok(post.clone())
    }

    async fn delete(&self, id: i64) -> Result<(), DomainError> {
        let mut posts = self.posts.lock().unwrap();
        let idx = posts
            .iter()
            .position(|p| p.id == id)
            .ok_or(DomainError::PostNotFound)?;
        posts.remove(idx);
        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        let posts = self.posts.lock().unwrap();
        let total = posts.len() as i64;
        let page: Vec<Post> = posts
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        Ok((page, total))
    }
}

pub struct MockUserRepository {
    users: Mutex<Vec<User>>,
    fail_create: bool,
}

impl MockUserRepository {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            users: Mutex::new(vec![]),
            fail_create: false,
        })
    }
    pub fn always_exists() -> Arc<Self> {
        Arc::new(Self {
            users: Mutex::new(vec![]),
            fail_create: true,
        })
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create(
        &self,
        req: &RegisterUserRequest,
        password_hash: &str,
    ) -> Result<User, DomainError> {
        if self.fail_create {
            return Err(DomainError::UserAlreadyExists);
        }
        let mut users = self.users.lock().unwrap();
        let id = users.len() as i64 + 1;
        let user = User {
            id,
            username: req.username.clone(),
            email: req.email.clone(),
            password_hash: password_hash.to_owned(),
            created_at: Utc::now(),
        };
        users.push(user.clone());
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.username == username)
            .cloned()
            .ok_or(DomainError::UserNotFound)
    }

    async fn find_by_id(&self, id: i64) -> Result<User, DomainError> {
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == id)
            .cloned()
            .ok_or(DomainError::UserNotFound)
    }
}
