use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::{
    data::user_repository::UserRepository,
    domain::{DomainError, LoginUserRequest, RegisterUserRequest, User},
    infrastructure::jwt::JwtService,
};

/// Application service for user registration and login.
pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    /// Create an `AuthService` backed by the given repository and JWT signer.
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repo,
            jwt_service,
        }
    }

    /// Register a new user, hash their password, and return a signed JWT token.
    pub async fn register(&self, req: &RegisterUserRequest) -> Result<(User, String), DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .to_string();

        let user = self.user_repo.create(req, &password_hash).await?;
        let token = self
            .jwt_service
            .generate_token(user.id, &user.username)
            .map_err(|e| DomainError::TokenError(e.to_string()))?;

        Ok((user, token))
    }

    /// Verify credentials and return a signed JWT token on success.
    pub async fn login(&self, req: &LoginUserRequest) -> Result<(User, String), DomainError> {
        let user = self
            .user_repo
            .find_by_username(&req.username)
            .await
            .map_err(|_| DomainError::InvalidCredentials)?;

        let parsed =
            PasswordHash::new(&user.password_hash).map_err(|_| DomainError::InvalidCredentials)?;
        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed)
            .map_err(|_| DomainError::InvalidCredentials)?;

        let token = self
            .jwt_service
            .generate_token(user.id, &user.username)
            .map_err(|e| DomainError::TokenError(e.to_string()))?;

        Ok((user, token))
    }
}
