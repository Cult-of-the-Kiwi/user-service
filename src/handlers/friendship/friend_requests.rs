use std::sync::Arc;

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{
            Error,
            domain::DomainError::InternalError,
            friend_request::FriendRequestError::{
                FriendRequestAlreadyExists, FriendRequestAlreadyHandled, FriendRequestDoesNotExist,
            },
            user::UserError::UserDoesNotExist,
        },
        models::friend_request::{FriendRequest, FriendRequestDirection, FriendRequestRange},
        types::UserID,
    },
};

pub async fn handle_request_friend(
    db: Arc<dyn UserRepository>,
    from_user_id: UserID,
    to_user_id: UserID,
) -> Result<(), Error> {
    let request = FriendRequest {
        from_user_id,
        to_user_id,
        created_at: None,
        state: Default::default(),
    };

    db.insert_friend_request(&request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            devcord_sqlx_utils::error::Error::AlreadyExists => FriendRequestAlreadyExists.into(),
            _ => InternalError.into(),
        })
}

pub async fn handle_accept_request(
    db: Arc<dyn UserRepository>,
    receiver_id: UserID,
    sender_id: UserID,
) -> Result<(), Error> {
    let mut request = FriendRequest {
        from_user_id: sender_id,
        to_user_id: receiver_id,
        created_at: None,
        state: Default::default(),
    };

    let mut existing = db.get_friend_request(&request).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => FriendRequestDoesNotExist.into(),
        _ => InternalError.into(),
    })?;

    if !existing.is_pending() {
        return Err(FriendRequestAlreadyHandled.into());
    }

    existing.accept();

    db.update_friend_request(&existing)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => FriendRequestDoesNotExist.into(),
            _ => InternalError.into(),
        })?;

    db.insert_friendship(&existing.from_user_id, &existing.to_user_id)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })
}

pub async fn handle_reject_request(
    db: Arc<dyn UserRepository>,
    receiver_id: UserID,
    sender_id: UserID,
) -> Result<(), Error> {
    let mut request = FriendRequest {
        from_user_id: sender_id,
        to_user_id: receiver_id,
        created_at: None,
        state: Default::default(),
    };

    let mut existing = db.get_friend_request(&request).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => FriendRequestDoesNotExist.into(),
        _ => InternalError.into(),
    })?;

    if !existing.is_pending() {
        return Err(FriendRequestAlreadyHandled.into());
    }

    existing.reject();

    db.update_friend_request(&existing)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => FriendRequestDoesNotExist.into(),
            _ => InternalError.into(),
        })
}

pub async fn handle_get_requests_sent(
    db: Arc<dyn UserRepository>,
    user_id: UserID,
    range: FriendRequestRange,
) -> Result<Vec<FriendRequest>, Error> {
    db.get_friend_requests(&user_id, &range, &FriendRequestDirection::Sent)
        .await
        .map_err(|_| InternalError.into())
}

pub async fn handle_get_requests_received(
    db: Arc<dyn UserRepository>,
    user_id: UserID,
    range: FriendRequestRange,
) -> Result<Vec<FriendRequest>, Error> {
    db.get_friend_requests(&user_id, &range, &FriendRequestDirection::Received)
        .await
        .map_err(|_| InternalError.into())
}
