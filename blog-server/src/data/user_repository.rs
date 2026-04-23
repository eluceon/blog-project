use async_trait::async_trait;

use crate::domain::{DomainError, RegisterUserRequest, User};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        req: &RegisterUserRequest,
        password_hash: &str,
    ) -> Result<User, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<User, DomainError>;
    #[allow(dead_code)]
    async fn find_by_id(&self, id: i64) -> Result<User, DomainError>;
}
