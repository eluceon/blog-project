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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(DomainError::UserNotFound.to_string(), "User not found");
        assert_eq!(
            DomainError::UserAlreadyExists.to_string(),
            "User already exists"
        );
        assert_eq!(
            DomainError::InvalidCredentials.to_string(),
            "Invalid credentials"
        );
        assert_eq!(DomainError::PostNotFound.to_string(), "Post not found");
        assert_eq!(
            DomainError::Forbidden.to_string(),
            "Forbidden: you are not the author"
        );
        assert_eq!(
            DomainError::Database("conn refused".into()).to_string(),
            "Database error: conn refused"
        );
        assert_eq!(
            DomainError::TokenError("expired".into()).to_string(),
            "Token error: expired"
        );
        assert_eq!(
            DomainError::Internal("oops".into()).to_string(),
            "Internal error: oops"
        );
    }
}
