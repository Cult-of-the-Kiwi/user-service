use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use super::json_error_response;

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
    fn into_response(self) -> Response {
        match self {
            BlockError::BlockDoesNotExist => {
                json_error_response(StatusCode::NOT_FOUND, self.to_string())
            }
            BlockError::BlockAlreadyExists => {
                json_error_response(StatusCode::CONFLICT, self.to_string())
            }
            BlockError::CannotBlockSelf | BlockError::CannotUnblockSelf => {
                json_error_response(StatusCode::BAD_REQUEST, self.to_string())
            }
        }
    }
}
