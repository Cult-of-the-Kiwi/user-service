pub(crate) mod auth;
pub(crate) mod block;
pub(crate) mod domain;
pub(crate) mod friend_request;
pub(crate) mod user;

use axum::{http::Response, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("User error: {0}")]
    User(user::UserError),
    #[error("Authentication error: {0}")]
    Auth(auth::AuthError),
    #[error("Domain error: {0}")]
    Domain(domain::DomainError),
    #[error("Friend request error: {0}")]
    FriendRequest(friend_request::FriendRequestError),
    #[error("Block error: {0}")]
    Block(block::BlockError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response<axum::body::Body> {
        match self {
            Error::User(e) => e.into_response(),
            Error::Auth(e) => e.into_response(),
            Error::Domain(e) => e.into_response(),
            Error::FriendRequest(e) => e.into_response(),
            Error::Block(e) => e.into_response(),
        }
    }
}

impl Into<Error> for user::UserError {
    fn into(self) -> Error {
        Error::User(self)
    }
}
impl Into<Error> for auth::AuthError {
    fn into(self) -> Error {
        Error::Auth(self)
    }
}
impl Into<Error> for domain::DomainError {
    fn into(self) -> Error {
        Error::Domain(self)
    }
}
impl Into<Error> for friend_request::FriendRequestError {
    fn into(self) -> Error {
        Error::FriendRequest(self)
    }
}
impl Into<Error> for block::BlockError {
    fn into(self) -> Error {
        Error::Block(self)
    }
}
