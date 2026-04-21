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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(BlogClientError::NotFound.to_string(), "Not found");
        assert_eq!(BlogClientError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(
            BlogClientError::InvalidRequest("bad id".into()).to_string(),
            "Invalid request: bad id"
        );
        assert_eq!(
            BlogClientError::NoToken.to_string(),
            "No token stored — please login first"
        );
        assert_eq!(
            BlogClientError::ServerError {
                status: 500,
                body: "oops".into()
            }
            .to_string(),
            "Server returned an error: 500 — oops"
        );
    }

    #[test]
    fn grpc_error_converts_from_status() {
        let status = tonic::Status::not_found("post not found");
        let err = BlogClientError::from(status);
        assert!(err.to_string().contains("not found"));
    }
}
