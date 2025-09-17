use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use devcord_middlewares::middlewares::auth::Authenticated;

use crate::{
    api_utils::{
        responses::{CONFLICT_UPDATING_PROFILE, INTERNAL_SERVER_ERROR, OK, USER_DOES_NOT_EXIST},
        structs::{UpdateUser, User},
    },
    app::AppState,
    sql_utils::calls::UserRepository,
};

pub async fn update<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Authenticated { claims, jwt: _ }: Authenticated,
    Json(request): Json<UpdateUser>,
) -> impl IntoResponse {
    match state.db.update_user(&request).await {
        Ok(_) => OK,
        Err(e) => match e {
            devcord_sqlx_utils::error::Error::AlreadyExists => CONFLICT_UPDATING_PROFILE,
            devcord_sqlx_utils::error::Error::RowNotFound => USER_DOES_NOT_EXIST,
            _ => INTERNAL_SERVER_ERROR,
        },
    }
}

pub async fn get_user<T: UserRepository>(
    State(state): State<Arc<AppState<T>>>,
    Json(user): Json<User>,
) -> impl IntoResponse {
    Json(state.db.get_user(&user.id).await)
}
