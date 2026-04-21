use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Token lifetime in seconds (24 hours).
const TOKEN_EXPIRY_SECS: u64 = 86_400;

/// Claims embedded in a JWT token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// ID of the authenticated user.
    pub user_id: i64,
    /// Username of the authenticated user.
    pub username: String,
    /// Unix timestamp (seconds) when the token expires.
    pub exp: usize,
}

/// Signs and verifies JWT tokens using a shared secret.
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    /// Create a `JwtService` from a base64 or plain-text secret string.
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Generate a signed JWT token for the given user, valid for 24 hours.
    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String> {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before UNIX_EPOCH")?
            .as_secs();

        let claims = Claims {
            user_id,
            username: username.to_owned(),
            exp: (now_secs + TOKEN_EXPIRY_SECS) as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key).context("failed to encode JWT")
    }

    /// Verify a JWT token and return its claims if valid.
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let data = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .context("invalid or expired token")?;
        Ok(data.claims)
    }
}
