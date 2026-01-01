use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum BlockError {
    #[error("Block does not exist")]
    BlockDoesNotExist,
    #[error("Block already exists")]
    BlockAlreadyExists,
    #[error("You cannot block yourself")]
    CannotBlockSelf,
    #[error("You cannot unblock yourself")]
    CannotUnblockSelf,
}

impl IntoResponse for BlockError {
    fn into_response(self) -> Response<Body> {
        match self {
            BlockError::BlockDoesNotExist => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
            BlockError::BlockAlreadyExists => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            BlockError::CannotBlockSelf | BlockError::CannotUnblockSelf => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}
