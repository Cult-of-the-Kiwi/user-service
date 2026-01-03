use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use super::json_error_response;

#[derive(Debug, Error)]
pub(crate) enum DomainError {
    #[error("Internal server error")]
    InternalError,
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        match self {
            DomainError::InternalError => {
                json_error_response(StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        }
    }
}
