use thiserror::Error;

/// Domain-level errors for the blog application.
#[derive(Debug, Error)]
pub enum DomainError {
    /// The requested user does not exist.
    #[error("User not found")]
    UserNotFound,

    /// A user with the same username or email already exists.
    #[error("User already exists")]
    UserAlreadyExists,

    /// The provided credentials are incorrect.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// The requested post does not exist.
    #[error("Post not found")]
    PostNotFound,

    /// The caller is not the author of the resource.
    #[error("Forbidden: you are not the author")]
    Forbidden,

    /// A database operation failed.
    #[error("Database error: {0}")]
    Database(String),

    /// A JWT token operation (generation or verification) failed.
    #[error("Token error: {0}")]
    TokenError(String),

    /// An unexpected internal error occurred.
    #[error("Internal error: {0}")]
    Internal(String),
}
