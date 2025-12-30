use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    domain::models::{block::Block, range::Range},
    handlers::block::{handle_block, handle_get_blocks, handle_unblock},
};

pub async fn block(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<Block>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    handle_block(state.db.clone(), request.from_user_id, request.to_user_id).await
}

pub async fn unblock(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<Block>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    handle_unblock(state.db.clone(), request.from_user_id, request.to_user_id).await
}

pub async fn get_blocks(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
    handle_get_blocks(state.db.clone(), claims.user_id, range)
        .await
        .map(Json)
}
