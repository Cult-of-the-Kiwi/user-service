use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AuthError {
    #[error("User is not authenticated")]
    NotAuthenticated,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response<axum::body::Body> {
        match self {
            AuthError::NotAuthenticated => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }
        }
    }
}
