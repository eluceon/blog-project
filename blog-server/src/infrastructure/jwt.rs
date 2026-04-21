use anyhow::{Context, Result};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
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

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-minimum-32-chars-pad!!";

    #[test]
    fn round_trip_preserves_claims() {
        let svc = JwtService::new(SECRET);
        let token = svc.generate_token(42, "alice").unwrap();
        let claims = svc.verify_token(&token).unwrap();
        assert_eq!(claims.user_id, 42);
        assert_eq!(claims.username, "alice");
        assert!(claims.exp > 0);
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let svc_a = JwtService::new("secret-a-minimum-32-chars-padded!!");
        let svc_b = JwtService::new("secret-b-minimum-32-chars-padded!!");
        let token = svc_a.generate_token(1, "bob").unwrap();
        assert!(svc_b.verify_token(&token).is_err());
    }

    #[test]
    fn tampered_token_is_rejected() {
        let svc = JwtService::new(SECRET);
        let token = svc.generate_token(1, "bob").unwrap();
        let tampered = format!("{token}tampered");
        assert!(svc.verify_token(&tampered).is_err());
    }
}
