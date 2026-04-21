use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("gRPC error: {0}")]
    Grpc(Box<tonic::Status>),

    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("No token stored — please login first")]
    NoToken,

    #[error("Server returned an error: {status} — {body}")]
    ServerError { status: u16, body: String },
}

pub type Result<T> = std::result::Result<T, BlogClientError>;

impl From<tonic::Status> for BlogClientError {
    fn from(s: tonic::Status) -> Self {
        BlogClientError::Grpc(Box::new(s))
    }
}
