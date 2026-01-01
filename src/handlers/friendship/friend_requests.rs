use std::sync::Arc;

use devcord_events::{
    events::{
        Event,
        user::{FriendRequestAnswered, FriendRequestCreated, UserEvent},
    },
    publisher::EventManager,
};

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{
            Error,
            domain::DomainError::InternalError,
            friend_request::FriendRequestError::{
                CannotAcceptSelf, CannotDeleteSelf, CannotRejectSelf, CannotSendToSelf,
                FriendRequestAlreadyExists, FriendRequestAlreadyHandled, FriendRequestDoesNotExist,
            },
            user::UserError::UserDoesNotExist,
        },
        models::friend_request::{
            FriendRequest, FriendRequestDirection, FriendRequestRange, FriendRequestState,
        },
        types::UserID,
    },
};

pub async fn handle_request_friend(
    db: &Arc<dyn UserRepository>,
    event_manager: Arc<dyn EventManager<Event = Event>>,
    from_user_id: UserID,
    to_user_id: UserID,
) -> Result<(), Error> {
    if from_user_id == to_user_id {
        return Err(CannotSendToSelf.into());
    }

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
        })?;

    let sender = db
        .get_user(&request.from_user_id)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })?;

    let event = Event::UserEvent(UserEvent::FriendRequestCreatedEvent(FriendRequestCreated {
        from_username: sender.username,
    }));
    event_manager
        .notify(event)
        .await
        .map_err(|_| InternalError.into())
}

pub async fn handle_accept_request(
    db: &Arc<dyn UserRepository>,
    event_manager: Arc<dyn EventManager<Event = Event>>,
    receiver_id: UserID,
    sender_id: UserID,
) -> Result<(), Error> {
    if receiver_id == sender_id {
        return Err(CannotAcceptSelf.into());
    }

    let request = FriendRequest {
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
        })?;

    let sender = db
        .get_user(&existing.from_user_id)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })?;

    let event = Event::UserEvent(UserEvent::FriendRequestAnsweredEvent(
        FriendRequestAnswered {
            from_username: sender.username,
            accepted: true,
        },
    ));
    event_manager
        .notify(event)
        .await
        .map_err(|_| InternalError.into())
}

pub async fn handle_reject_request(
    db: &Arc<dyn UserRepository>,
    event_manager: Arc<dyn EventManager<Event = Event>>,
    receiver_id: UserID,
    sender_id: UserID,
) -> Result<(), Error> {
    if receiver_id == sender_id {
        return Err(CannotRejectSelf.into());
    }

    let request = FriendRequest {
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
        })?;

    let sender = db
        .get_user(&existing.from_user_id)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => UserDoesNotExist.into(),
            _ => InternalError.into(),
        })?;

    let event = Event::UserEvent(UserEvent::FriendRequestAnsweredEvent(
        FriendRequestAnswered {
            from_username: sender.username,
            accepted: false,
        },
    ));
    event_manager
        .notify(event)
        .await
        .map_err(|_| InternalError.into())
}

pub async fn handle_delete_request(
    db: &Arc<dyn UserRepository>,
    sender_id: UserID,
    recipient_id: UserID,
) -> Result<(), Error> {
    if sender_id == recipient_id {
        return Err(CannotDeleteSelf.into());
    }

    let request = FriendRequest {
        from_user_id: sender_id,
        to_user_id: recipient_id,
        created_at: None,
        state: FriendRequestState::Pending,
    };

    db.delete_friend_request(&request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => FriendRequestDoesNotExist.into(),
            _ => InternalError.into(),
        })
}

pub async fn handle_get_requests_sent(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    range: FriendRequestRange,
) -> Result<Vec<FriendRequest>, Error> {
    db.get_friend_requests(&user_id, &range, &FriendRequestDirection::Sent)
        .await
        .map_err(|_| InternalError.into())
}

pub async fn handle_get_requests_received(
    db: &Arc<dyn UserRepository>,
    user_id: UserID,
    range: FriendRequestRange,
) -> Result<Vec<FriendRequest>, Error> {
    db.get_friend_requests(&user_id, &range, &FriendRequestDirection::Received)
        .await
        .map_err(|_| InternalError.into())
}
