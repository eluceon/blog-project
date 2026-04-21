use std::sync::Arc;

use actix_web::{
    HttpMessage, HttpRequest, HttpResponse, Responder,
    error::{
        ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized,
    },
    web,
};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};

use crate::{
    application::{AuthService, BlogService},
    domain::{
        CreatePostRequest, DomainError, LoginUserRequest, RegisterUserRequest, UpdatePostRequest,
    },
    presentation::middleware::AuthenticatedUser,
};

/// Map a domain error to the corresponding HTTP error response.
fn map_domain_error(e: DomainError) -> actix_web::Error {
    match e {
        DomainError::UserNotFound | DomainError::PostNotFound => ErrorNotFound(e.to_string()),
        DomainError::UserAlreadyExists => ErrorConflict(e.to_string()),
        DomainError::InvalidCredentials => ErrorUnauthorized(e.to_string()),
        DomainError::Forbidden => ErrorForbidden(e.to_string()),
        other => {
            error!("Internal error: {other}");
            ErrorInternalServerError("Internal server error")
        }
    }
}

/// Extract the authenticated user inserted by the JWT middleware.
fn get_auth_user(req: &HttpRequest) -> Result<AuthenticatedUser, actix_web::Error> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("Not authenticated"))
}

// ─── Auth endpoints ───────────────────────────────────────────────────────────

/// `POST /api/auth/register` — register a new user and return a JWT token.
pub async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<RegisterUserRequest>,
) -> Result<impl Responder, actix_web::Error> {
    info!("POST /api/auth/register username={}", body.username);
    let (user, token) = auth_service
        .register(&body.0)
        .await
        .map_err(map_domain_error)?;
    Ok(HttpResponse::Created().json(json!({ "token": token, "user": user })))
}

/// `POST /api/auth/login` — verify credentials and return a JWT token.
pub async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    body: web::Json<LoginUserRequest>,
) -> Result<impl Responder, actix_web::Error> {
    info!("POST /api/auth/login username={}", body.username);
    let (user, token) = auth_service
        .login(&body.0)
        .await
        .map_err(map_domain_error)?;
    Ok(HttpResponse::Ok().json(json!({ "token": token, "user": user })))
}

// ─── Post endpoints ───────────────────────────────────────────────────────────

/// `POST /api/posts` — create a new post (requires authentication).
pub async fn create_post(
    req: HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    body: web::Json<CreatePostRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let auth_user = get_auth_user(&req)?;
    info!("POST /api/posts user_id={}", auth_user.user_id);
    let post = blog_service
        .create_post(&body.0, auth_user.user_id)
        .await
        .map_err(map_domain_error)?;
    Ok(HttpResponse::Created().json(post))
}

/// `GET /api/posts/{id}` — retrieve a single post by ID (public).
pub async fn get_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> Result<impl Responder, actix_web::Error> {
    let id = path.into_inner();
    info!("GET /api/posts/{id}");
    let post = blog_service.get_post(id).await.map_err(map_domain_error)?;
    Ok(HttpResponse::Ok().json(post))
}

/// `PUT /api/posts/{id}` — update a post (requires authentication; caller must be author).
pub async fn update_post(
    req: HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
    body: web::Json<UpdatePostRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let auth_user = get_auth_user(&req)?;
    let id = path.into_inner();
    info!("PUT /api/posts/{id} user_id={}", auth_user.user_id);
    let post = blog_service
        .update_post(id, &body.0, auth_user.user_id)
        .await
        .map_err(map_domain_error)?;
    Ok(HttpResponse::Ok().json(post))
}

/// `DELETE /api/posts/{id}` — delete a post (requires authentication; caller must be author).
pub async fn delete_post(
    req: HttpRequest,
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> Result<impl Responder, actix_web::Error> {
    let auth_user = get_auth_user(&req)?;
    let id = path.into_inner();
    info!("DELETE /api/posts/{id} user_id={}", auth_user.user_id);
    blog_service
        .delete_post(id, auth_user.user_id)
        .await
        .map_err(map_domain_error)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Query parameters for the post list endpoint.
#[derive(Deserialize)]
pub struct ListQuery {
    /// Maximum number of posts to return (defaults to 10).
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of posts to skip (defaults to 0).
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    10
}

/// `GET /api/posts` — paginated list of all posts (public).
pub async fn list_posts(
    blog_service: web::Data<Arc<BlogService>>,
    query: web::Query<ListQuery>,
) -> Result<impl Responder, actix_web::Error> {
    info!(
        "GET /api/posts limit={} offset={}",
        query.limit, query.offset
    );
    let (posts, total) = blog_service
        .list_posts(query.limit, query.offset)
        .await
        .map_err(map_domain_error)?;
    Ok(HttpResponse::Ok().json(json!({
        "posts": posts,
        "total": total,
        "limit": query.limit,
        "offset": query.offset,
    })))
}
