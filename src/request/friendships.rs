use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use devcord_middlewares::middlewares::auth::Authenticated;
use sqlx::PgPool;

use crate::{
    api_utils::structs::{FriendRequest, FriendRequestDirection, FriendRequestRange, Range},
    app::AppState,
    sql_utils::calls::UserRepository,
};

pub async fn request_friend(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    if state
        .db
        .get_user_friend(&claims.user_id, &request.to_user_id)
        .await
        .is_some()
    {
        todo!("Is already friend error");
    }

    if state
        .db
        .get_user_block(&request.to_user_id, &claims.user_id)
        .await
        .is_some()
    {
        todo!("Error because block");
    }

    request.from_user_id = claims.user_id;

    state
        .db
        .insert_friend_request(&request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::AlreadyExists => todo!("Request already exists"),
            devcord_sqlx_utils::error::Error::ForeignKeyViolation => {
                todo!("Other user or user does not exist")
            }
            devcord_sqlx_utils::error::Error::CheckViolation => {
                todo!("Internal error (We dont have checks on this")
            }
            devcord_sqlx_utils::error::Error::InternalError => todo!("Internal error"),
            _ => todo!(),
        });

    todo!("Add ok response")
}

pub async fn accept_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    request.accept();

    match update_request(&request, &state.db).await {
        Ok(_) => todo!("Okay"),
        Err(e) => return e,
    };

    match state
        .db
        .insert_friendship(&claims.user_id, &request.to_user_id)
        .await
    {
        Ok(_) => todo!("Return ok"),
        Err(_) => todo!(),
    }
}

pub async fn reject_request(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id;
    request.reject();

    match update_request(&request, &state.db).await {
        Ok(_) => todo!("Okay"),
        Err(e) => return e,
    }

    match state
        .db
        .insert_friendship(&claims.user_id, request.to_user_id)
        .await {}
}

async fn update_request(request: &FriendRequest, db: &PgPool) -> Result<(), impl IntoResponse> {
    db.update_friend_request(request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::ForeignKeyViolation => {
                todo!("Other user or user does not exist")
            }
            devcord_sqlx_utils::error::Error::RowNotFound => todo!("Request does not exist"),
            _ => todo!("Internal server error"),
        })?;

    Ok(())
}

async fn get_requests_sent(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> impl IntoResponse {
    todo!("Check request limits and stuff");

    let requests = state
        .db
        .get_friend_requests(&claims.user_id, &range, &FriendRequestDirection::Sent)
        .await;

    todo!("Return requests");
}

async fn get_requests_received(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
    todo!("Check request limits and stuff");

    let requests = state
        .db
        .get_friend_requests(&claims.user_id, &range, &FriendRequestDirection::Received)
        .await;

    todo!("Return requests");
}

async fn get_friends(
    State(state): State<Arc<AppState>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
    let friends = state.db.get_user_friends(&claims.user_id, &range).await;

    todo!("Return values")
}
