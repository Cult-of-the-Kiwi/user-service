use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    app::AppState,
    application::repositories::user_repository::UserRepository,
    domain::models::{update_user::UpdateUser, user::User},
};

pub async fn update<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(request): Json<UpdateUser>,
) -> impl IntoResponse {
}

pub async fn get_user<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Json(user): Json<User>,
) -> impl IntoResponse {
}
