use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum DomainError {
    #[error("Internal server error")]
    InternalError,
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response<Body> {
        match self {
            DomainError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }
    }
}
