use std::sync::Arc;

use axum::{
    Json, Router,
    routing::{delete, get, post, put},
};
use serde_json::json;

use crate::{
    app::AppState,
    application::controllers::{
        block::{block, get_blocks, is_blocked, unblock},
        friendship::{
            friend_request::{
                accept_request, delete_request, get_requests_received, get_requests_sent,
                reject_request, request_friend,
            },
            friends::{get_friends, is_friend, remove_friend},
        },
        user::{get_user, update_user},
    },
};

pub(crate) fn new() -> Router<Arc<AppState>> {
    let user_requests_router = Router::new()
        .route("/", post(request_friend).delete(delete_request))
        .route("/accept", put(accept_request))
        .route("/reject", put(reject_request));

    let requests_router = Router::new()
        .nest("/{user}", user_requests_router)
        .route("/received", get(get_requests_received))
        .route("/sent", get(get_requests_sent));

    let friends_router = Router::new()
        .route("/", get(get_friends))
        .route("/{user}", delete(remove_friend));

    let user_router = Router::new()
        .route("/is-friend", get(is_friend))
        .route("/is-blocked", get(is_blocked))
        .route("/", get(get_user));

    let block_router = Router::new()
        .route("/{user}", post(block).delete(unblock))
        .route("/", get(get_blocks));

    let friendships_router = Router::new()
        .nest("/requests", requests_router)
        .nest("/friends", friends_router)
        .route("/", get(get_friends));

    Router::new()
        .nest("/{user}", user_router)
        .nest("/friendship", friendships_router)
        .nest("/blocks", block_router)
        .route("/update", post(update_user))
        .route("/health", get(Json(json!({ "status": "ok" }))))
}
