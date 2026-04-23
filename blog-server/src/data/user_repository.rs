use async_trait::async_trait;
use sqlx::PgPool;

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

#[derive(sqlx::FromRow)]
struct UserRow {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        User {
            id: r.id,
            username: r.username,
            email: r.email,
            password_hash: r.password_hash,
            created_at: r.created_at,
        }
    }
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(
        &self,
        req: &RegisterUserRequest,
        password_hash: &str,
    ) -> Result<User, DomainError> {
        sqlx::query_as::<_, UserRow>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)
             RETURNING id, username, email, password_hash, created_at",
        )
        .bind(&req.username)
        .bind(&req.email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.is_unique_violation() => {
                DomainError::UserAlreadyExists
            }
            _ => DomainError::Database(e.to_string()),
        })
    }

    async fn find_by_username(&self, username: &str) -> Result<User, DomainError> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DomainError::UserNotFound,
            _ => DomainError::Database(e.to_string()),
        })
    }

    async fn find_by_id(&self, id: i64) -> Result<User, DomainError> {
        sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DomainError::UserNotFound,
            _ => DomainError::Database(e.to_string()),
        })
    }
}
