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
    application::repositories::user_repository::UserRepository,
    domain::models::friend_request::{FriendRequest, FriendRequestRange},
};

pub async fn request_friend<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
}

pub async fn accept_request<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
}

pub async fn reject_request<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
}

pub async fn get_requests_sent<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Either<Json<Option<Vec<FriendRequest>>>, impl IntoResponse> {
}

pub async fn get_requests_received<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Either<Json<Option<Vec<FriendRequest>>>, impl IntoResponse> {
}
