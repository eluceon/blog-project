use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::{
    application::{AuthService, BlogService},
    domain::{
        CreatePostRequest, DomainError, LoginUserRequest, RegisterUserRequest, UpdatePostRequest,
    },
    infrastructure::jwt::JwtService,
    proto::blog::{
        self, AuthResponse, DeletePostRequest, DeletePostResponse, GetPostRequest,
        ListPostsRequest, ListPostsResponse, LoginRequest, Post as ProtoPost, PostResponse,
        RegisterRequest, UpdatePostRequest as ProtoUpdatePostRequest,
        blog_service_server::BlogService as BlogServiceTrait,
    },
};

/// Map a domain error to the corresponding gRPC status code.
fn domain_to_status(e: DomainError) -> Status {
    match e {
        DomainError::UserNotFound | DomainError::PostNotFound => Status::not_found(e.to_string()),
        DomainError::UserAlreadyExists => Status::already_exists(e.to_string()),
        DomainError::InvalidCredentials => Status::unauthenticated(e.to_string()),
        DomainError::Forbidden => Status::permission_denied(e.to_string()),
        DomainError::TokenError(msg) => Status::unauthenticated(msg),
        other => {
            error!("gRPC internal error: {other}");
            Status::internal("Internal server error")
        }
    }
}

/// Extract the Bearer token from gRPC request metadata.
// tonic::Status is 176 bytes and cannot be boxed here — it is tonic's own return type.
#[allow(clippy::result_large_err)]
fn extract_bearer_token<T>(req: &Request<T>) -> Result<&str, Status> {
    req.metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization header"))
}

/// Convert a domain `Post` to its protobuf representation.
fn post_to_proto(p: crate::domain::Post) -> ProtoPost {
    ProtoPost {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        author_username: p.author_username,
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }
}

/// gRPC service implementation that delegates to the application layer.
#[derive(Clone)]
pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    /// Create the service, sharing the same application services as the HTTP handlers.
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }
}

#[tonic::async_trait]
impl BlogServiceTrait for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC Register username={}", req.username);
        let domain_req = RegisterUserRequest {
            username: req.username,
            email: req.email,
            password: req.password,
        };
        let (user, token) = self
            .auth_service
            .register(&domain_req)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(AuthResponse {
            token,
            user: Some(blog::User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            }),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC Login username={}", req.username);
        let domain_req = LoginUserRequest {
            username: req.username,
            password: req.password,
        };
        let (user, token) = self
            .auth_service
            .login(&domain_req)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(AuthResponse {
            token,
            user: Some(blog::User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at.to_rfc3339(),
            }),
        }))
    }

    async fn create_post(
        &self,
        request: Request<blog::CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let token = extract_bearer_token(&request)?;
        let claims = self
            .jwt_service
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        let req = request.into_inner();
        info!("gRPC CreatePost user_id={}", claims.user_id);
        let domain_req = CreatePostRequest {
            title: req.title,
            content: req.content,
        };
        let post = self
            .blog_service
            .create_post(&domain_req, claims.user_id)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(PostResponse {
            post: Some(post_to_proto(post)),
        }))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let id = request.into_inner().id;
        info!("gRPC GetPost id={id}");
        let post = self
            .blog_service
            .get_post(id)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(PostResponse {
            post: Some(post_to_proto(post)),
        }))
    }

    async fn update_post(
        &self,
        request: Request<ProtoUpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let token = extract_bearer_token(&request)?;
        let claims = self
            .jwt_service
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        let req = request.into_inner();
        info!("gRPC UpdatePost id={} user_id={}", req.id, claims.user_id);
        let domain_req = UpdatePostRequest {
            title: if req.title.is_empty() {
                None
            } else {
                Some(req.title)
            },
            content: if req.content.is_empty() {
                None
            } else {
                Some(req.content)
            },
        };
        let post = self
            .blog_service
            .update_post(req.id, &domain_req, claims.user_id)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(PostResponse {
            post: Some(post_to_proto(post)),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let token = extract_bearer_token(&request)?;
        let claims = self
            .jwt_service
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        let id = request.into_inner().id;
        info!("gRPC DeletePost id={id} user_id={}", claims.user_id);
        self.blog_service
            .delete_post(id, claims.user_id)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(DeletePostResponse { success: true }))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit > 0 { req.limit as i64 } else { 10 };
        let offset = req.offset as i64;
        info!("gRPC ListPosts limit={limit} offset={offset}");

        let (posts, total) = self
            .blog_service
            .list_posts(limit, offset)
            .await
            .map_err(domain_to_status)?;

        Ok(Response::new(ListPostsResponse {
            posts: posts.into_iter().map(post_to_proto).collect(),
            total,
            limit: limit as i32,
            offset: offset as i32,
        }))
    }
}
