use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

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
    fn into_response(self) -> Response<Body> {
        match self {
            FriendRequestError::FriendRequestDoesNotExist => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            FriendRequestError::FriendRequestAlreadyHandled
            | FriendRequestError::FriendRequestAlreadyExists => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            FriendRequestError::CannotSendToSelf
            | FriendRequestError::CannotAcceptSelf
            | FriendRequestError::CannotRejectSelf
            | FriendRequestError::CannotDeleteSelf => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}
