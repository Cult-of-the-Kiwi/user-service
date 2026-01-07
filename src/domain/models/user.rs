use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::types::{Time, UserID, Username};

#[derive(FromRow, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct User {
    pub username: Username,
    pub id: UserID,
    pub created_at: Option<Time>,
}

impl IntoResponse for User {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
