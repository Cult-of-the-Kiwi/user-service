use std::sync::Arc;

use devcord_events::{
    events::{
        Event,
        user::{UserEvent, UserUpdated},
    },
    publisher::EventManager,
};
use tracing::{debug, error};

use crate::{
    application::repositories::user_repository::UserRepository,
    domain::{
        errors::{Error, domain::DomainError::InternalError, user::UserError::UserDoesNotExist},
        models::{update_user::UpdateUser, user::User},
        types::UserID,
    },
};

pub async fn handle_update_user(
    db: &Arc<dyn UserRepository>,
    event_manager: Arc<dyn EventManager<Event = Event>>,
    request: UpdateUser,
    user_id: UserID,
) -> Result<(), Error> {
    db.update_user(&user_id, &request)
        .await
        .map_err(|e| match e {
            devcord_sqlx_utils::error::Error::RowNotFound => {
                debug!(?user_id, "User not found");
                Error::from(UserDoesNotExist)
            }
            e => {
                error!(?e, "Failed to update user");
                Error::from(InternalError)
            }
        })?;

    let event = Event::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated { id: user_id }));
    event_manager.notify(event).await.map_err(|e| {
        error!(?e, "Failed to publish UserUpdated event");
        InternalError.into()
    })
}

pub async fn handle_get_user(db: &Arc<dyn UserRepository>, user_id: UserID) -> Result<User, Error> {
    let result = db.get_user(&user_id).await.map_err(|e| match e {
        devcord_sqlx_utils::error::Error::RowNotFound => {
            debug!(?user_id, "User not found");
            Error::from(UserDoesNotExist)
        }
        e => {
            error!(?e, "Failed to fetch user");
            Error::from(InternalError)
        }
    })?;

    Ok(result)
}
