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
        }
    }
}
