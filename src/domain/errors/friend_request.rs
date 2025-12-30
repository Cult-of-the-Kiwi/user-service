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
        }
    }
}
