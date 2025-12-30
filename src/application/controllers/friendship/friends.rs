use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    domain::models::{range::Range, user::User},
    handlers::friendship::friends::{handle_get_friends, handle_is_friend, handle_remove_friend},
};

pub async fn get_friends(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    handle_get_friends(state.db.clone(), claims.user_id, range)
        .await
        .map(Json)
}

pub async fn remove_friend(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(user): Json<User>,
) -> impl IntoResponse {
    handle_remove_friend(state.db.clone(), claims.user_id, user.id).await
}

pub async fn is_friend(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(user): Json<User>,
) -> impl IntoResponse {
    handle_is_friend(state.db.clone(), claims.user_id, user.id)
        .await
        .map(Json)
}
