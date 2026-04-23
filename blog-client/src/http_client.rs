use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::{BlogClientError, Result};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct UserData {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserData,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PostData {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<PostData>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub struct HttpBlogClient {
    client: Client,
    base_url: String,
}

impl HttpBlogClient {
    pub fn new(base_url: String) -> crate::error::Result<Self> {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(BlogClientError::Http)?;
        Ok(Self { client, base_url })
    }

    async fn check_response(&self, resp: reqwest::Response) -> Result<reqwest::Response> {
        let status = resp.status();
        match status {
            s if s.is_success() => Ok(resp),
            StatusCode::NOT_FOUND => Err(BlogClientError::NotFound),
            StatusCode::UNAUTHORIZED => Err(BlogClientError::Unauthorized),
            _ => {
                let body = resp.text().await.unwrap_or_default();
                Err(BlogClientError::ServerError {
                    status: status.as_u16(),
                    body,
                })
            }
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse> {
        let resp = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "email": email,
                "password": password,
            }))
            .send()
            .await?;

        Ok(self
            .check_response(resp)
            .await?
            .json::<AuthResponse>()
            .await?)
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse> {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;

        Ok(self
            .check_response(resp)
            .await?
            .json::<AuthResponse>()
            .await?)
    }

    pub async fn create_post(&self, token: &str, title: &str, content: &str) -> Result<PostData> {
        let resp = self
            .client
            .post(format!("{}/api/posts", self.base_url))
            .bearer_auth(token)
            .json(&serde_json::json!({ "title": title, "content": content }))
            .send()
            .await?;

        Ok(self.check_response(resp).await?.json::<PostData>().await?)
    }

    pub async fn get_post(&self, id: i64) -> Result<PostData> {
        let resp = self
            .client
            .get(format!("{}/api/posts/{id}", self.base_url))
            .send()
            .await?;

        Ok(self.check_response(resp).await?.json::<PostData>().await?)
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<PostData> {
        let resp = self
            .client
            .put(format!("{}/api/posts/{id}", self.base_url))
            .bearer_auth(token)
            .json(&serde_json::json!({ "title": title, "content": content }))
            .send()
            .await?;

        Ok(self.check_response(resp).await?.json::<PostData>().await?)
    }

    pub async fn delete_post(&self, token: &str, id: i64) -> Result<()> {
        let resp = self
            .client
            .delete(format!("{}/api/posts/{id}", self.base_url))
            .bearer_auth(token)
            .send()
            .await?;

        self.check_response(resp).await?;
        Ok(())
    }

    pub async fn list_posts(&self, limit: i32, offset: i32) -> Result<ListPostsResponse> {
        let resp = self
            .client
            .get(format!(
                "{}/api/posts?limit={limit}&offset={offset}",
                self.base_url
            ))
            .send()
            .await?;

        Ok(self
            .check_response(resp)
            .await?
            .json::<ListPostsResponse>()
            .await?)
    }
}
