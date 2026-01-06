use std::sync::Arc;

use tracing::{debug, error};

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{
            Error, domain::DomainError::InternalError,
            friendship::FriendshipError::CannotRemoveSelf, user::UserError::UserDoesNotExist,
        },
        models::{friendship::Friendship, range::Range, user::User},
        types::UserID,
    },
};

pub async fn handle_get_friends(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    range: Range,
) -> Result<Vec<User>, Error> {
    db.get_user_friends(&user_id, &range)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => {
                debug!(?user_id, "User not found while fetching friends");
                UserDoesNotExist.into()
            }
            e => {
                error!(?e, "Failed to fetch friends");
                InternalError.into()
            },
        })
}

pub async fn handle_remove_friend(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    friend_id: UserID,
) -> Result<(), Error> {
    if user_id == friend_id {
        return Err(CannotRemoveSelf.into());
    }

    let friendship = Friendship {
        from_user_id: user_id.clone(),
        to_user_id: friend_id.clone(),
        created_at: None,
    };
    db.delete_friendship(&friendship)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => {
                debug!(?user_id, ?friend_id, "Friendship does not exist");
                UserDoesNotExist.into()
            }
            e => {
                error!(?e, "Failed to delete friendship");
                InternalError.into()
            },
        })
}

pub async fn handle_is_friend(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    friend_id: UserID,
) -> Result<bool, Error> {
    match db.get_user_if_friend(&user_id, &friend_id).await {
        Ok(_) => Ok(true),
        Err(devcord_sqlx_utils::error::Error::RowNotFound) => Ok(false),
        Err(e) => {
            error!(?e, "Failed to check friendship status");
            Err(InternalError.into())
        }
    }
}
