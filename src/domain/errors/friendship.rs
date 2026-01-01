use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FriendshipError {
    #[error("You cannot remove yourself as a friend")]
    CannotRemoveSelf,
}

impl IntoResponse for FriendshipError {
    fn into_response(self) -> Response<Body> {
        match self {
            FriendshipError::CannotRemoveSelf => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}
