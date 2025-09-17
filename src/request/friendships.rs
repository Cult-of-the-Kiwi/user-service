use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::either::Either::{self, E1, E2};
use devcord_middlewares::middlewares::auth::Authenticated;
use sqlx::PgPool;

use crate::{
    api_utils::{
        responses::{
            ALREADY_FRIEND, ApiResponse, ApiResponseMessage, INTERNAL_SERVER_ERROR, OK,
            RANGE_TOO_LARGE, REQUEST_ALREADY_EXIST, REQUEST_DOES_NOT_EXIST, USER_DOES_NOT_EXIST,
        },
        structs::{FriendRequest, FriendRequestDirection, FriendRequestRange, Range},
    },
    app::AppState,
    sql_utils::calls::UserRepository,
};

//FIXME! (Lamoara) change this and bellow fn for less repeated code
pub async fn request_friend<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    if state
        .db
        .get_user_friend(&claims.user_id, &request.to_user_id)
        .await
        .is_some()
    {
        return ALREADY_FRIEND;
    }

    if state
        .db
        .get_user_block(&request.to_user_id, &claims.user_id)
        .await
        .is_some()
    {
        return USER_DOES_NOT_EXIST;
    }

    request.from_user_id = claims.user_id;

    match state.db.insert_friend_request(&request).await {
        Ok(_) => OK,
        Err(e) => match e {
            devcord_sqlx_utils::error::Error::AlreadyExists => REQUEST_ALREADY_EXIST,
            devcord_sqlx_utils::error::Error::ForeignKeyViolation => {
                return USER_DOES_NOT_EXIST;
            }
            _ => INTERNAL_SERVER_ERROR,
        },
    }
}

pub async fn accept_request<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id.clone();
    request.accept();

    match update_request(&request, &state.db).await {
        Ok(_) => (),
        Err(e) => return e,
    };

    match state
        .db
        .insert_friendship(&claims.user_id, &request.to_user_id)
        .await
    {
        Ok(_) => OK,
        Err(e) => match e {
            devcord_sqlx_utils::error::Error::AlreadyExists => ALREADY_FRIEND,
            _ => INTERNAL_SERVER_ERROR,
        },
    }
}

pub async fn reject_request<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(mut request): Json<FriendRequest>,
) -> impl IntoResponse {
    request.from_user_id = claims.user_id.clone();
    request.reject();

    match update_request(&request, &state.db).await {
        Ok(_) => (),
        Err(e) => return e,
    };

    match state
        .db
        .insert_friendship(&claims.user_id, &request.to_user_id)
        .await
    {
        Ok(_) => OK,
        Err(e) => match e {
            devcord_sqlx_utils::error::Error::AlreadyExists => ALREADY_FRIEND,
            _ => INTERNAL_SERVER_ERROR,
        },
    }
}

pub async fn update_request<T: UserRepository>(
    request: &FriendRequest,
    db: &T,
) -> Result<(), ApiResponse<ApiResponseMessage>> {
    db.update_friend_request(request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::ForeignKeyViolation => USER_DOES_NOT_EXIST,
            devcord_sqlx_utils::error::Error::RowNotFound => REQUEST_DOES_NOT_EXIST,
            _ => INTERNAL_SERVER_ERROR,
        })?;

    Ok(())
}

pub async fn get_requests_sent<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Either<Json<Option<Vec<FriendRequest>>>, impl IntoResponse> {
    //FIXME! (Lamoara) Make range limit env or something
    if (range.to - range.from) > 10 {
        return E2(RANGE_TOO_LARGE);
    }

    let requests = state
        .db
        .get_friend_requests(&claims.user_id, &range, &FriendRequestDirection::Sent)
        .await;

    E1(Json(requests))
}

pub async fn get_requests_received<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<FriendRequestRange>,
) -> Either<Json<Option<Vec<FriendRequest>>>, impl IntoResponse> {
    //FIXME! (Lamoara) Make range limit env or something
    if (range.to - range.from) > 10 {
        return E2(RANGE_TOO_LARGE);
    }

    let requests = state
        .db
        .get_friend_requests(&claims.user_id, &range, &FriendRequestDirection::Received)
        .await;

    E1(Json(requests))
}

pub async fn get_friends<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Query(range): Query<Range>,
) -> impl IntoResponse {
    Json(state.db.get_user_friends(&claims.user_id, &range).await)
}
