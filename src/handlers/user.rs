use std::sync::Arc;

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{Error, domain::DomainError::InternalError, user::UserError::UserDoesNotExist},
        models::{update_user::UpdateUser, user::User},
        types::UserID,
    },
};

pub async fn handle_update_user(
    db: Arc<dyn UserRepository>,
    request: UpdateUser,
    user_id: UserID,
) -> Result<(), Error> {
    db.update_user(&user_id, &request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })
}

pub async fn handle_get_user(db: Arc<dyn UserRepository>, user_id: UserID) -> Result<User, Error> {
    db.get_user(&user_id).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
        _ => InternalError.into(),
    })
}
