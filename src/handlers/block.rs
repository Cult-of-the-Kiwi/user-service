use std::sync::Arc;

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{
            block::BlockError::{BlockAlreadyExists, BlockDoesNotExist},
            domain::DomainError::InternalError,
            user::UserError::UserDoesNotExist,
            Error,
        },
        models::{block::Block, range::Range},
        types::UserID,
    },
};

pub async fn handle_block(
    db: Arc<dyn UserRepository>,
    from_user_id: UserID,
    to_user_id: UserID,
) -> Result<(), Error> {
    let request = Block {
        from_user_id,
        to_user_id,
        created_at: None,
    };

    db.insert_block(&request).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
        devcord_sqlx_utils::error::Error::AlreadyExists => BlockAlreadyExists.into(),
        _ => InternalError.into(),
    })
}

pub async fn handle_unblock(
    db: Arc<dyn UserRepository>,
    from_user_id: UserID,
    to_user_id: UserID,
) -> Result<(), Error> {
    let request = Block {
        from_user_id,
        to_user_id,
        created_at: None,
    };

    db.delete_block(&request).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => BlockDoesNotExist.into(),
        _ => InternalError.into(),
    })
}

pub async fn handle_get_blocks(
    db: Arc<dyn UserRepository>,
    user_id: UserID,
    range: Range,
) -> Result<Vec<Block>, Error> {
    db.get_user_blocks(&user_id, &range)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })
}
