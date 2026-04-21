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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, web};
    use actix_web_httpauth::middleware::HttpAuthentication;
    use serde_json::Value;

    use super::*;
    use crate::{
        application::{AuthService, BlogService},
        data::{PostRepository, UserRepository},
        domain::RegisterUserRequest,
        infrastructure::jwt::JwtService,
        presentation::middleware::jwt_validator,
        test_mocks::{MockPostRepository, MockUserRepository, make_jwt, make_post},
    };

    /// Build a test-mode actix service with the same route layout as main.rs.
    macro_rules! init_app {
        ($auth_svc:expr, $blog_svc:expr, $jwt:expr) => {{
            let auth_mw = HttpAuthentication::bearer(jwt_validator);
            actix_web::test::init_service(
                App::new()
                    .app_data(web::Data::new($auth_svc))
                    .app_data(web::Data::new($blog_svc))
                    .app_data(web::Data::new($jwt))
                    .service(
                        web::scope("/api/auth")
                            .route("/register", web::post().to(register))
                            .route("/login", web::post().to(login)),
                    )
                    .service(
                        web::scope("/api/posts")
                            .route("", web::get().to(list_posts))
                            .route("/{id}", web::get().to(get_post))
                            .service(
                                web::scope("")
                                    .wrap(auth_mw)
                                    .route("", web::post().to(create_post))
                                    .route("/{id}", web::put().to(update_post))
                                    .route("/{id}", web::delete().to(delete_post)),
                            ),
                    ),
            )
            .await
        }};
    }

    fn make_services(
        user_repo: Arc<dyn UserRepository>,
        post_repo: Arc<dyn PostRepository>,
        jwt: Arc<JwtService>,
    ) -> (Arc<AuthService>, Arc<BlogService>) {
        (
            Arc::new(AuthService::new(user_repo, jwt)),
            Arc::new(BlogService::new(post_repo)),
        )
    }

    #[actix_web::test]
    async fn register_returns_201_with_token_and_user() {
        let jwt = make_jwt();
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::empty(),
            jwt.clone(),
        );
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::post()
            .uri("/api/auth/register")
            .set_json(serde_json::json!({
                "username": "alice", "email": "a@a.com", "password": "secret123"
            }))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 201);
        let body: Value = actix_web::test::read_body_json(resp).await;
        assert!(body["token"].is_string());
        assert_eq!(body["user"]["username"], "alice");
    }

    #[actix_web::test]
    async fn login_returns_200_with_valid_credentials() {
        let jwt = make_jwt();
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::empty(),
            jwt.clone(),
        );
        // Seed a user via the service layer so the argon2 hash is stored.
        auth.register(&RegisterUserRequest {
            username: "bob".to_owned(),
            email: "b@b.com".to_owned(),
            password: "pass123".to_owned(),
        })
        .await
        .unwrap();
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(serde_json::json!({"username": "bob", "password": "pass123"}))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 200);
        let body: Value = actix_web::test::read_body_json(resp).await;
        assert!(body["token"].is_string());
    }

    #[actix_web::test]
    async fn login_returns_401_on_wrong_password() {
        let jwt = make_jwt();
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::empty(),
            jwt.clone(),
        );
        auth.register(&RegisterUserRequest {
            username: "carol".to_owned(),
            email: "c@c.com".to_owned(),
            password: "right".to_owned(),
        })
        .await
        .unwrap();
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(serde_json::json!({"username": "carol", "password": "wrong"}))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn list_posts_returns_200_with_posts_array() {
        let jwt = make_jwt();
        let posts = vec![make_post(1, 1), make_post(2, 1)];
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::with(posts),
            jwt.clone(),
        );
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::get()
            .uri("/api/posts")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 200);
        let body: Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body["total"], 2);
        assert!(body["posts"].is_array());
    }

    #[actix_web::test]
    async fn get_post_returns_404_when_not_found() {
        let jwt = make_jwt();
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::empty(),
            jwt.clone(),
        );
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::get()
            .uri("/api/posts/999")
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 404);
    }

    #[actix_web::test]
    async fn create_post_without_token_returns_401() {
        let jwt = make_jwt();
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::empty(),
            jwt.clone(),
        );
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::post()
            .uri("/api/posts")
            .set_json(serde_json::json!({"title": "T", "content": "C"}))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn create_post_with_valid_token_returns_201() {
        let jwt = make_jwt();
        let token = jwt.generate_token(1, "dave").unwrap();
        let (auth, blog) = make_services(
            MockUserRepository::empty(),
            MockPostRepository::empty(),
            jwt.clone(),
        );
        let app = init_app!(auth, blog, jwt);

        let req = actix_web::test::TestRequest::post()
            .uri("/api/posts")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(serde_json::json!({"title": "Hello", "content": "World"}))
            .to_request();
        let resp = actix_web::test::call_service(&app, req).await;

        assert_eq!(resp.status().as_u16(), 201);
        let body: Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body["title"], "Hello");
        assert_eq!(body["author_id"], 1);
    }
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
