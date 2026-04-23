use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Post not found")]
    PostNotFound,

    #[error("Forbidden: you are not the author")]
    Forbidden,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Token error: {0}")]
    TokenError(String),

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
