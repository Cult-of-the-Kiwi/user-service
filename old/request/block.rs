use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    api_utils::{
        responses::{
            BLOCK_ALREADY_EXISTS, BLOCK_DOES_NOT_EXISTS, INTERNAL_SERVER_ERROR, OK,
            USER_DOES_NOT_EXIST,
        },
        structs::{Block, Range},
    },
    app::AppState,
    sql_utils::calls::UserRepository,
};

pub async fn block<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<Block>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    match state.db.insert_block(&request).await {
        Ok(_) => OK,
        Err(e) => match e {
            devcord_sqlx_utils::error::Error::AlreadyExists => BLOCK_ALREADY_EXISTS,
            devcord_sqlx_utils::error::Error::ForeignKeyViolation => USER_DOES_NOT_EXIST,
            _ => INTERNAL_SERVER_ERROR,
        },
    }
}

pub async fn unblock<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<Block>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    match state.db.delete_block(&request).await {
        Ok(_) => OK,
        Err(e) => match e {
            devcord_sqlx_utils::error::Error::RowNotFound => BLOCK_DOES_NOT_EXISTS,
            _ => INTERNAL_SERVER_ERROR,
        },
    }
}

pub async fn get_blocks<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
    Json(state.db.get_user_blocks(&claims.user_id, &range).await)
}
