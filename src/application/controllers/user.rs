use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    domain::{models::update_user::UpdateUser, types::UserID},
    handlers::user::{handle_get_user, handle_update_user},
};

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(request): Json<UpdateUser>,
) -> impl IntoResponse {
    handle_update_user(
        &state.db,
        state.event_manager.clone(),
        request,
        claims.user_id,
    )
    .await
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user): Path<UserID>,
) -> impl IntoResponse {
    handle_get_user(&state.db, user).await
}
