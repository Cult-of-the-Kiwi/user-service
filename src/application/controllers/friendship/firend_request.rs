use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::either::Either;
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    domain::models::friend_request::{FriendRequest, FriendRequestRange},
    handlers::friendship::friend_requests::{
        handle_accept_request, handle_get_requests_received, handle_get_requests_sent,
        handle_reject_request, handle_request_friend,
    },
};

pub async fn request_friend(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    handle_request_friend(state.db.clone(), request.from_user_id, request.to_user_id).await
}

pub async fn accept_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(request): Json<FriendRequest>,
) -> impl IntoResponse {
    let sender_id = request.to_user_id;
    handle_accept_request(state.db.clone(), claims.user_id, sender_id).await
}

pub async fn reject_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(request): Json<FriendRequest>,
) -> impl IntoResponse {
    let sender_id = request.to_user_id;
    handle_reject_request(state.db.clone(), claims.user_id, sender_id).await
}

pub async fn get_requests_sent(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Either<Json<Option<Vec<FriendRequest>>>, impl IntoResponse> {
    match handle_get_requests_sent(state.db.clone(), claims.user_id, range).await {
        Ok(reqs) => Either::E1(Json(Some(reqs))),
        Err(err) => Either::E2(err),
    }
}

pub async fn get_requests_received(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Either<Json<Option<Vec<FriendRequest>>>, impl IntoResponse> {
    match handle_get_requests_received(state.db.clone(), claims.user_id, range).await {
        Ok(reqs) => Either::E1(Json(Some(reqs))),
        Err(err) => Either::E2(err),
    }
}
