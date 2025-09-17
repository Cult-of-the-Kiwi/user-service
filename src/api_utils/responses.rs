use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize, Clone, Copy)]
pub struct ApiResponseMessage {
    pub message: &'static str,
}

pub type ApiResponse<T> = (StatusCode, Json<T>);

pub static INTERNAL_SERVER_ERROR: ApiResponse<ApiResponseMessage> = (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(ApiResponseMessage {
        message: "Fluvio error",
    }),
);

pub static USER_DOES_NOT_EXIST: ApiResponse<ApiResponseMessage> = (
    StatusCode::NOT_FOUND,
    Json(ApiResponseMessage {
        message: "User does not exist",
    }),
);

pub static REQUEST_DOES_NOT_EXIST: ApiResponse<ApiResponseMessage> = (
    StatusCode::NOT_FOUND,
    Json(ApiResponseMessage {
        message: "Request does not exist",
    }),
);

pub static REQUEST_ALREADY_EXIST: ApiResponse<ApiResponseMessage> = (
    StatusCode::CONFLICT,
    Json(ApiResponseMessage {
        message: "Request already exist",
    }),
);

pub static REQUEST_NOT_PENDING: ApiResponse<ApiResponseMessage> = (
    StatusCode::CONFLICT,
    Json(ApiResponseMessage {
        message: "Friend request already answered",
    }),
);

pub static BLOCK_ALREADY_EXISTS: ApiResponse<ApiResponseMessage> = (
    StatusCode::CONFLICT,
    Json(ApiResponseMessage {
        message: "Block already exists",
    }),
);

pub static BLOCK_DOES_NOT_EXISTS: ApiResponse<ApiResponseMessage> = (
    StatusCode::NOT_FOUND,
    Json(ApiResponseMessage {
        message: "Block does not exist",
    }),
);

pub static ALREADY_FRIEND: ApiResponse<ApiResponseMessage> = (
    StatusCode::CONFLICT,
    Json(ApiResponseMessage {
        message: "User is already your friend",
    }),
);

pub static RANGE_TOO_LARGE: ApiResponse<ApiResponseMessage> = (
    StatusCode::BAD_REQUEST,
    Json(ApiResponseMessage {
        message: "Range is to big",
    }),
);

pub static CONFLICT_UPDATING_PROFILE: ApiResponse<ApiResponseMessage> = (
    StatusCode::CONFLICT,
    Json(ApiResponseMessage {
        message: "Profile values already taken",
    }),
);

pub static OK: ApiResponse<ApiResponseMessage> =
    (StatusCode::OK, Json(ApiResponseMessage { message: "Ok" }));
