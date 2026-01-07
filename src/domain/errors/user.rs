use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use super::json_error_response;

#[derive(Debug, Error)]
pub(crate) enum UserError {
    #[error("User does not exist")]
    UserDoesNotExist,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        match self {
            UserError::UserDoesNotExist => {
                json_error_response(StatusCode::NOT_FOUND, self.to_string())
            }
        }
    }
}
