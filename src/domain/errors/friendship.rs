use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use super::json_error_response;

#[derive(Debug, Error)]
pub(crate) enum FriendshipError {
    #[error("You cannot remove yourself as a friend")]
    CannotRemoveSelf,
}

impl IntoResponse for FriendshipError {
    fn into_response(self) -> Response {
        match self {
            FriendshipError::CannotRemoveSelf => {
                json_error_response(StatusCode::BAD_REQUEST, self.to_string())
            }
        }
    }
}
