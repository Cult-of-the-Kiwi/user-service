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
    domain::models::{range::Range, user::User},
};

pub async fn get_friends<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
}

pub async fn remove_friend<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(user): Json<User>,
) -> impl IntoResponse {
}
