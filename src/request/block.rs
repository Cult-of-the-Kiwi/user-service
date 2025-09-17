use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{api_utils::structs::Block, app::AppState};

pub async fn block(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(request): Json<Block>,
) -> impl IntoResponse {
}
