use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    app::AppState,
    application::controllers::{
        block::{block, get_blocks, unblock},
        friendship::{
            firend_request::{
                accept_request, get_requests_received, get_requests_sent, reject_request,
                request_friend,
            },
            friends::{get_friends, remove_friend},
        },
        user::{get_user, update_user},
    },
};

pub(crate) fn new() -> Router<Arc<AppState>> {
    let friendships_router = Router::new()
        .route("/request", post(request_friend))
        .route("/accept", post(accept_request))
        .route("/reject", post(reject_request))
        .route("/sent", get(get_requests_sent))
        .route("/received", get(get_requests_received))
        .route("/remove", post(remove_friend))
        .route("/", get(get_friends));

    let block_router = Router::new()
        .route("/block", post(block))
        .route("/unblock", post(unblock))
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
