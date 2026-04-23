use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{DomainError, Post};

use super::post_repository::PostRepository;

#[derive(sqlx::FromRow)]
struct PostRow {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    author_username: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<PostRow> for Post {
    fn from(r: PostRow) -> Self {
        Post {
            id: r.id,
            title: r.title,
            content: r.content,
            author_id: r.author_id,
            author_username: r.author_username,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        sqlx::query_as::<_, PostRow>(
            "WITH inserted AS (
                INSERT INTO posts (title, content, author_id)
                VALUES ($1, $2, $3)
                RETURNING id, title, content, author_id, created_at, updated_at
             )
             SELECT i.id, i.title, i.content, i.author_id,
                    u.username AS author_username, i.created_at, i.updated_at
             FROM inserted i
             JOIN users u ON u.id = i.author_id",
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| DomainError::Database(e.to_string()))
    }

    async fn find_by_id(&self, id: i64) -> Result<Post, DomainError> {
        sqlx::query_as::<_, PostRow>(
            "SELECT p.id, p.title, p.content, p.author_id,
                    u.username AS author_username, p.created_at, p.updated_at
             FROM posts p
             JOIN users u ON u.id = p.author_id
             WHERE p.id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DomainError::PostNotFound,
            _ => DomainError::Database(e.to_string()),
        })
    }

    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, DomainError> {
        sqlx::query_as::<_, PostRow>(
            "WITH updated AS (
                UPDATE posts
                SET title = COALESCE($1, title),
                    content = COALESCE($2, content),
                    updated_at = NOW()
                WHERE id = $3
                RETURNING id, title, content, author_id, created_at, updated_at
             )
             SELECT u2.id, u2.title, u2.content, u2.author_id,
                    usr.username AS author_username, u2.created_at, u2.updated_at
             FROM updated u2
             JOIN users usr ON usr.id = u2.author_id",
        )
        .bind(title)
        .bind(content)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DomainError::PostNotFound,
            _ => DomainError::Database(e.to_string()),
        })
    }

    async fn delete(&self, id: i64) -> Result<(), DomainError> {
        let result = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::PostNotFound);
        }
        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        let rows = sqlx::query_as::<_, PostRow>(
            "SELECT p.id, p.title, p.content, p.author_id,
                    u.username AS author_username, p.created_at, p.updated_at
             FROM posts p
             JOIN users u ON u.id = p.author_id
             ORDER BY p.created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok((rows.into_iter().map(Into::into).collect(), total))
    }
}
