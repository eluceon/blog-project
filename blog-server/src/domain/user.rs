use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A registered user stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier.
    pub id: i64,
    /// Unique username chosen at registration.
    pub username: String,
    /// User's email address.
    pub email: String,
    /// Argon2-hashed password — never serialized to API responses.
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// When the user account was created.
    pub created_at: DateTime<Utc>,
}

/// Request body for registering a new user.
#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    /// Desired username.
    pub username: String,
    /// Email address.
    pub email: String,
    /// Plain-text password (hashed before storage).
    pub password: String,
}

/// Request body for logging in.
#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    /// Username of the account to log into.
    pub username: String,
    /// Plain-text password for verification.
    pub password: String,
}
