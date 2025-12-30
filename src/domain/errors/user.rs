use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum UserError {
    #[error("User does not exist")]
    UserDoesNotExist,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response<Body> {
        match self {
            UserError::UserDoesNotExist => {
                (StatusCode::NOT_FOUND, self.to_string()).into_response()
            }
        }
    }
}
