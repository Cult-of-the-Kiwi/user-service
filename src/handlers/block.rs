use std::sync::Arc;

use tracing::{debug, error};

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{
            Error,
            block::BlockError::{
                BlockAlreadyExists, BlockDoesNotExist, CannotBlockSelf, CannotUnblockSelf,
            },
            domain::DomainError::InternalError,
            user::UserError::UserDoesNotExist,
        },
        models::{block::Block, range::Range},
        types::UserID,
    },
};

pub async fn handle_block(
    db: &Arc<dyn UserRepository>,
    from_user_id: UserID,
    to_user_id: UserID,
) -> Result<(), Error> {
    if from_user_id == to_user_id {
        return Err(CannotBlockSelf.into());
    }

    let request = Block {
        from_user_id,
        to_user_id,
        created_at: None,
    };

    db.insert_block(&request).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => {
            debug!(?request.from_user_id, ?request.to_user_id, "Cannot block, user not found");
            UserDoesNotExist.into()
        }
        devcord_sqlx_utils::error::Error::AlreadyExists => {
            debug!(?request.from_user_id, ?request.to_user_id, "Block already exists");
            BlockAlreadyExists.into()
        }
        e => {
            error!(?e, "Failed to insert block");
            InternalError.into()
        },
    })
}

pub async fn handle_unblock(
    db: &Arc<dyn UserRepository>,
    from_user_id: UserID,
    to_user_id: UserID,
) -> Result<(), Error> {
    if from_user_id == to_user_id {
        return Err(CannotUnblockSelf.into());
    }

    let request = Block {
        from_user_id,
        to_user_id,
        created_at: None,
    };

    db.delete_block(&request).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => {
            debug!(?request.from_user_id, ?request.to_user_id, "Block does not exist");
            BlockDoesNotExist.into()
        }
        e => {
            error!(?e, "Failed to delete block");
            InternalError.into()
        },
    })
}

pub async fn handle_get_blocks(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    range: Range,
) -> Result<Vec<Block>, Error> {
    db.get_user_blocks(&user_id, &range)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => {
                debug!(?user_id, "User not found while fetching blocks");
                UserDoesNotExist.into()
            }
            e => {
                error!(?e, "Failed to get user blocks");
                InternalError.into()
            },
        })
}

pub async fn handle_is_blocked(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    blocked_id: UserID,
) -> Result<bool, Error> {
    match db.get_user_if_blocked(&user_id, &blocked_id).await {
        Ok(_) => Ok(true),
        Err(devcord_sqlx_utils::error::Error::RowNotFound) => Ok(false),
        Err(e) => {
            error!(?e, "Failed to check if user is blocked");
            Err(InternalError.into())
        }
    }
}
