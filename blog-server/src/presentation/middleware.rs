use std::sync::Arc;

use actix_web::{
    dev::ServiceRequest,
    error::ErrorUnauthorized,
    web::Data,
    Error, HttpMessage,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::infrastructure::jwt::JwtService;

/// Carries the authenticated user identity through the request extension.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    #[allow(dead_code)]
    pub username: String,
}

/// actix-web-httpauth validator that checks JWT Bearer tokens.
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = req
        .app_data::<Data<Arc<JwtService>>>()
        .map(|d| d.get_ref().clone());

    let jwt_service = match jwt_service {
        Some(s) => s,
        None => {
            return Err((ErrorUnauthorized("JWT service unavailable"), req));
        }
    };

    match jwt_service.verify_token(credentials.token()) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthenticatedUser {
                user_id: claims.user_id,
                username: claims.username,
            });
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("Invalid or expired token"), req)),
    }
}
