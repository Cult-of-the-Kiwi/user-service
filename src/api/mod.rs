use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    app::AppState,
    application::controllers::{
        block::{block, get_blocks, is_blocked, unblock},
        friendship::{
            firend_request::{
                accept_request, delete_request, get_requests_received, get_requests_sent,
                reject_request, request_friend,
            },
            friends::{get_friends, is_friend, remove_friend},
        },
        user::{get_user, update_user},
    },
};

pub(crate) fn new() -> Router<Arc<AppState>> {
    let friendships_router = Router::new()
        .route("/request", post(request_friend))
        .route("/accept", post(accept_request))
        .route("/reject", post(reject_request))
        .route("/delete", post(delete_request))
        .route("/sent", get(get_requests_sent))
        .route("/received", get(get_requests_received))
        .route("/remove", post(remove_friend))
        .route("/is_friend", post(is_friend))
        .route("/", get(get_friends));

    let block_router = Router::new()
        .route("/block", post(block))
        .route("/unblock", post(unblock))
        .route("/is_blocked", post(is_blocked))
        .route("/", get(get_blocks));

    Router::new()
        .nest("/friendship", friendships_router)
        .nest("/blocks", block_router)
        .route("/update", post(update_user))
        .route("/", get(get_user))
        .route(
            "/health",
            get(|| async { "Long life to the allmighty turbofish" }),
        )
}
