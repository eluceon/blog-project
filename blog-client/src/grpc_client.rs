use tonic::{metadata::MetadataValue, transport::Channel, Request};

use crate::error::{BlogClientError, Result};
use crate::http_client::{AuthResponse, ListPostsResponse, PostData, UserData};
use crate::proto::blog::{
    blog_service_client::BlogServiceClient, CreatePostRequest, DeletePostRequest, GetPostRequest,
    ListPostsRequest, LoginRequest, RegisterRequest, UpdatePostRequest,
};

fn post_from_proto(p: crate::proto::blog::Post) -> PostData {
    PostData {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        author_username: p.author_username,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

/// gRPC transport implementation for `BlogClient`.
pub struct GrpcBlogClient {
    inner: BlogServiceClient<Channel>,
}

impl GrpcBlogClient {
    /// Connect to the gRPC server at `endpoint`.
    pub async fn new(endpoint: String) -> Result<Self> {
        let channel = Channel::from_shared(endpoint)
            .map_err(|e| BlogClientError::InvalidRequest(e.to_string()))?
            .connect()
            .await?;
        Ok(Self {
            inner: BlogServiceClient::new(channel),
        })
    }

    fn with_token<T>(&self, msg: T, token: &str) -> Result<Request<T>> {
        let mut req = Request::new(msg);
        let val = MetadataValue::try_from(format!("Bearer {token}"))
            .map_err(|e| BlogClientError::InvalidRequest(e.to_string()))?;
        req.metadata_mut().insert("authorization", val);
        Ok(req)
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse> {
        let resp = self
            .inner
            .register(RegisterRequest {
                username: username.to_owned(),
                email: email.to_owned(),
                password: password.to_owned(),
            })
            .await?
            .into_inner();

        let user = resp.user.ok_or_else(|| BlogClientError::ServerError {
            status: 0,
            body: "missing user in auth response".to_owned(),
        })?;
        Ok(AuthResponse {
            token: resp.token,
            user: UserData {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at,
            },
        })
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<AuthResponse> {
        let resp = self
            .inner
            .login(LoginRequest {
                username: username.to_owned(),
                password: password.to_owned(),
            })
            .await?
            .into_inner();

        let user = resp.user.ok_or_else(|| BlogClientError::ServerError {
            status: 0,
            body: "missing user in auth response".to_owned(),
        })?;
        Ok(AuthResponse {
            token: resp.token,
            user: UserData {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at,
            },
        })
    }

    pub async fn create_post(&mut self, token: &str, title: &str, content: &str) -> Result<PostData> {
        let req = self.with_token(
            CreatePostRequest {
                title: title.to_owned(),
                content: content.to_owned(),
            },
            token,
        )?;
        let post = self
            .inner
            .create_post(req)
            .await?
            .into_inner()
            .post
            .ok_or(BlogClientError::NotFound)?;
        Ok(post_from_proto(post))
    }

    pub async fn get_post(&mut self, id: i64) -> Result<PostData> {
        let post = self
            .inner
            .get_post(GetPostRequest { id })
            .await?
            .into_inner()
            .post
            .ok_or(BlogClientError::NotFound)?;
        Ok(post_from_proto(post))
    }

    pub async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<PostData> {
        let req = self.with_token(
            UpdatePostRequest {
                id,
                title: title.unwrap_or("").to_owned(),
                content: content.unwrap_or("").to_owned(),
            },
            token,
        )?;
        let post = self
            .inner
            .update_post(req)
            .await?
            .into_inner()
            .post
            .ok_or(BlogClientError::NotFound)?;
        Ok(post_from_proto(post))
    }

    pub async fn delete_post(&mut self, token: &str, id: i64) -> Result<()> {
        let req = self.with_token(DeletePostRequest { id }, token)?;
        self.inner.delete_post(req).await?;
        Ok(())
    }

    pub async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<ListPostsResponse> {
        let resp = self
            .inner
            .list_posts(ListPostsRequest { limit, offset })
            .await?
            .into_inner();
        Ok(ListPostsResponse {
            posts: resp.posts.into_iter().map(post_from_proto).collect(),
            total: resp.total,
            limit: resp.limit as i64,
            offset: resp.offset as i64,
        })
    }
}
