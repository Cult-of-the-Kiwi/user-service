use std::sync::Arc;

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{Error, domain::DomainError::InternalError, user::UserError::UserDoesNotExist},
        models::{friendship::Friendship, range::Range, user::User},
        types::UserID,
    },
};

pub async fn handle_get_friends(
    db: Arc<dyn UserRepository>,
    user_id: UserID,
    range: Range,
) -> Result<Vec<User>, Error> {
    db.get_user_friends(&user_id, &range)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })
}

pub async fn handle_remove_friend(
    db: Arc<dyn UserRepository>,
    user_id: UserID,
    friend_id: UserID,
) -> Result<(), Error> {
    let friendship = Friendship {
        from_user_id: user_id.clone(),
        to_user_id: friend_id.clone(),
        created_at: None,
    };
    db.delete_friendship(&friendship)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })
}
