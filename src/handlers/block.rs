use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    application::repositories::user_repository::UserRepository,
    domain::models::{block::Block, range::Range},
};

pub async fn block<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<Block>,
) -> impl IntoResponse {
}

pub async fn unblock<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<Block>,
) -> impl IntoResponse {
}

pub async fn get_blocks<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
}
