use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    domain::{
        models::{block::Block, range::Range},
        types::UserID,
    },
    handlers::block::{handle_block, handle_get_blocks, handle_is_blocked, handle_unblock},
};

pub async fn block(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    let request = Block::new(claims.user_id, user);
    handle_block(&state.db, request.from_user_id, request.to_user_id).await
}

pub async fn unblock(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    let request = Block::new(claims.user_id, user);
    handle_unblock(&state.db, request.from_user_id, request.to_user_id).await
}

pub async fn get_blocks(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
    handle_get_blocks(&state.db, claims.user_id, range)
        .await
        .map(Json)
}

pub async fn is_blocked(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    handle_is_blocked(&state.db, claims.user_id, user)
        .await
        .map(Json)
}
