use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    domain::{models::friend_request::FriendRequestRange, types::UserID},
    handlers::friendship::friend_requests::{
        handle_accept_request, handle_delete_request, handle_get_requests_received,
        handle_get_requests_sent, handle_reject_request, handle_request_friend,
    },
};

pub async fn request_friend(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    handle_request_friend(&state.db, state.event_manager.clone(), claims.user_id, user).await
}

pub async fn accept_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    handle_accept_request(&state.db, state.event_manager.clone(), claims.user_id, user).await
}

pub async fn reject_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    handle_reject_request(&state.db, state.event_manager.clone(), claims.user_id, user).await
}

pub async fn delete_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    handle_delete_request(&state.db, claims.user_id, user).await
}

pub async fn get_requests_sent(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    handle_get_requests_sent(&state.db, claims.user_id, range)
        .await
        .map(Json)
}

pub async fn get_requests_received(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    handle_get_requests_received(&state.db, claims.user_id, range)
        .await
        .map(Json)
}
