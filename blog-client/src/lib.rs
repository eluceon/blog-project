pub mod error;
pub mod grpc_client;
pub mod http_client;

mod proto {
    pub mod blog {
        tonic::include_proto!("blog");
    }
}

pub use error::{BlogClientError, Result};
pub use http_client::{AuthResponse, ListPostsResponse, PostData, UserData};

/// Transport selection for the blog client
#[derive(Debug, Clone)]
pub enum Transport {
    /// HTTP REST API; value is the base URL, e.g. `"http://localhost:8080"`
    Http(String),
    /// gRPC API; value is the endpoint URL, e.g. `"http://localhost:50051"`
    Grpc(String),
}

/// Unified client for the blog service, supporting both HTTP and gRPC transports.
pub struct BlogClient {
    transport: Transport,
    http: Option<http_client::HttpBlogClient>,
    grpc: Option<grpc_client::GrpcBlogClient>,
    token: Option<String>,
}

impl BlogClient {
    /// Create and connect a new client.  For gRPC this establishes a channel.
    pub async fn new(transport: Transport) -> Result<Self> {
        match &transport {
            Transport::Http(url) => Ok(Self {
                http: Some(http_client::HttpBlogClient::new(url.clone())?),
                grpc: None,
                token: None,
                transport,
            }),
            Transport::Grpc(endpoint) => {
                let grpc = grpc_client::GrpcBlogClient::new(endpoint.clone()).await?;
                Ok(Self {
                    http: None,
                    grpc: Some(grpc),
                    token: None,
                    transport,
                })
            }
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn require_token(&self) -> Result<&str> {
        self.token
            .as_deref()
            .ok_or(BlogClientError::NoToken)
    }

    pub async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<AuthResponse> {
        let resp = match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.register(username, email, password).await?,
            (_, Some(g)) => g.register(username, email, password).await?,
            _ => unreachable!("BlogClient must have exactly one transport"),
        };
        self.token = Some(resp.token.clone());
        Ok(resp)
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<AuthResponse> {
        let resp = match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.login(username, password).await?,
            (_, Some(g)) => g.login(username, password).await?,
            _ => unreachable!(),
        };
        self.token = Some(resp.token.clone());
        Ok(resp)
    }

    pub async fn create_post(&mut self, title: &str, content: &str) -> Result<PostData> {
        let token = self.require_token()?.to_owned();
        match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.create_post(&token, title, content).await,
            (_, Some(g)) => g.create_post(&token, title, content).await,
            _ => unreachable!(),
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<PostData> {
        match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.get_post(id).await,
            (_, Some(g)) => g.get_post(id).await,
            _ => unreachable!(),
        }
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<PostData> {
        let token = self.require_token()?.to_owned();
        match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.update_post(&token, id, title, content).await,
            (_, Some(g)) => g.update_post(&token, id, title, content).await,
            _ => unreachable!(),
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<()> {
        let token = self.require_token()?.to_owned();
        match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.delete_post(&token, id).await,
            (_, Some(g)) => g.delete_post(&token, id).await,
            _ => unreachable!(),
        }
    }

    pub async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<ListPostsResponse> {
        match (&self.http, &mut self.grpc) {
            (Some(h), _) => h.list_posts(limit, offset).await,
            (_, Some(g)) => g.list_posts(limit, offset).await,
            _ => unreachable!(),
        }
    }

    pub fn transport(&self) -> &Transport {
        &self.transport
    }
}
