pub(crate) mod block;
pub(crate) mod domain;
pub(crate) mod friend_request;
pub(crate) mod friendship;
pub(crate) mod user;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("User error: {0}")]
    User(user::UserError),
    #[error("Domain error: {0}")]
    Domain(domain::DomainError),
    #[error("Friend request error: {0}")]
    FriendRequest(friend_request::FriendRequestError),
    #[error("Block error: {0}")]
    Block(block::BlockError),
    #[error("Friendship error: {0}")]
    Friendship(friendship::FriendshipError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::User(e) => e.into_response(),
            Error::Domain(e) => e.into_response(),
            Error::FriendRequest(e) => e.into_response(),
            Error::Block(e) => e.into_response(),
            Error::Friendship(e) => e.into_response(),
        }
    }
}

impl From<user::UserError> for Error {
    fn from(val: user::UserError) -> Self {
        Error::User(val)
    }
}
impl From<domain::DomainError> for Error {
    fn from(val: domain::DomainError) -> Self {
        Error::Domain(val)
    }
}
impl From<friend_request::FriendRequestError> for Error {
    fn from(val: friend_request::FriendRequestError) -> Self {
        Error::FriendRequest(val)
    }
}
impl From<block::BlockError> for Error {
    fn from(val: block::BlockError) -> Self {
        Error::Block(val)
    }
}
impl From<friendship::FriendshipError> for Error {
    fn from(val: friendship::FriendshipError) -> Self {
        Error::Friendship(val)
    }
}

pub(crate) fn json_error_response(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}
