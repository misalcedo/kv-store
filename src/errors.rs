use axum::response::Response;
use axum::{body, http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug)]
pub enum KeyValueError {
    #[error("Unable to find key '{0}'.")]
    NotFound(String),
    #[error("Unable to perform Redis operation: {0}.")]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for KeyValueError {
    fn into_response(self) -> Response {
        let status_code = match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = body::boxed(body::Full::from(self.to_string()));

        Response::builder().status(status_code).body(body).unwrap()
    }
}
