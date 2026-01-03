use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use super::json_error_response;

#[derive(Debug, Error)]
pub(crate) enum FriendRequestError {
    #[error("Friend request does not exist")]
    FriendRequestDoesNotExist,
    #[error("Friend request already handled")]
    FriendRequestAlreadyHandled,
    #[error("Friend request already exists")]
    FriendRequestAlreadyExists,
    #[error("You cannot send a friend request to yourself")]
    CannotSendToSelf,
    #[error("You cannot accept a friend request to yourself")]
    CannotAcceptSelf,
    #[error("You cannot reject a friend request to yourself")]
    CannotRejectSelf,
    #[error("You cannot delete a friend request to yourself")]
    CannotDeleteSelf,
}

impl IntoResponse for FriendRequestError {
    fn into_response(self) -> Response {
        match self {
            FriendRequestError::FriendRequestDoesNotExist => {
                json_error_response(StatusCode::NOT_FOUND, self.to_string())
            }
            FriendRequestError::FriendRequestAlreadyHandled
            | FriendRequestError::FriendRequestAlreadyExists => {
                json_error_response(StatusCode::CONFLICT, self.to_string())
            }
            FriendRequestError::CannotSendToSelf
            | FriendRequestError::CannotAcceptSelf
            | FriendRequestError::CannotRejectSelf
            | FriendRequestError::CannotDeleteSelf => {
                json_error_response(StatusCode::BAD_REQUEST, self.to_string())
            }
        }
    }
}
