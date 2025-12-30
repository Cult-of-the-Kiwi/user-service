use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use devcord_events::{
    events::{
        Event,
        user::{FriendRequestAnswered, FriendRequestCreated, UserEvent, UserUpdated},
    },
    publisher::EventManager,
};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::{
    application::repositories::user_repository::UserRepository,
    handlers::{
        friendship::friend_requests::{
            handle_accept_request, handle_reject_request, handle_request_friend,
        },
        user::handle_update_user,
    },
    tests::{
        in_memory_event_manager::InMemoryEventManager,
        in_memory_user_repository::{InMemoryUserRepository, sample_users},
    },
};

async fn subscribe_event(
    event_manager: &Arc<dyn EventManager<Event = Event>>,
    event: Event,
) -> anyhow::Result<mpsc::Receiver<Event>> {
    let (tx, rx) = mpsc::channel::<Event>(1);
    event_manager
        .subscribe(
            event,
            Box::new(move |event| {
                let tx = tx.clone();
                Box::pin(async move {
                    tx.send(event).await.unwrap();
                })
            }),
        )
        .await?;

    Ok(rx)
}

async fn expect_event(rx: &mut mpsc::Receiver<Event>) -> anyhow::Result<Event> {
    timeout(Duration::from_secs(5), rx.recv())
        .await?
        .ok_or_else(|| anyhow!("event not received"))
}

async fn new_event_manager() -> anyhow::Result<Arc<dyn EventManager<Event = Event>>> {
    Ok(Arc::new(InMemoryEventManager::new()))
}

#[tokio::test]
async fn test_sends_user_updated_event() -> anyhow::Result<()> {
    let repo = Arc::new(InMemoryUserRepository::new(sample_users()));
    let event_manager = new_event_manager().await?;

    let expected_event = Event::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: "user-1".to_string(),
    }));
    let mut rx = subscribe_event(&event_manager, expected_event.clone()).await?;

    handle_update_user(
        repo,
        event_manager.clone(),
        crate::domain::models::update_user::UpdateUser {
            username: Some("alice-updated".to_string()),
        },
        "user-1".to_string(),
    )
    .await
    .unwrap();

    let received = expect_event(&mut rx).await?;
    assert_eq!(expected_event, received);

    Ok(())
}

#[tokio::test]
async fn test_sends_friend_request_created_event() -> anyhow::Result<()> {
    let repo = Arc::new(InMemoryUserRepository::new(sample_users()));
    let event_manager = new_event_manager().await?;

    let expected_event =
        Event::UserEvent(UserEvent::FriendRequestCreatedEvent(FriendRequestCreated {
            from_username: "alice".to_string(),
        }));
    let mut rx = subscribe_event(&event_manager, expected_event.clone()).await?;

    handle_request_friend(
        repo,
        event_manager.clone(),
        "user-1".to_string(),
        "user-2".to_string(),
    )
    .await
    .unwrap();

    let received = expect_event(&mut rx).await?;
    assert_eq!(expected_event, received);

    Ok(())
}

#[tokio::test]
async fn test_sends_friend_request_answered_event_on_accept() -> anyhow::Result<()> {
    let repo = Arc::new(InMemoryUserRepository::new(sample_users()));
    repo.insert_friend_request(&crate::domain::models::friend_request::FriendRequest {
        from_user_id: "user-1".to_string(),
        to_user_id: "user-2".to_string(),
        created_at: None,
        state: crate::domain::models::friend_request::FriendRequestState::Pending,
    })
    .await
    .unwrap();

    let event_manager = new_event_manager().await?;
    let expected_event = Event::UserEvent(UserEvent::FriendRequestAnsweredEvent(
        FriendRequestAnswered {
            from_username: "alice".to_string(),
            accepted: true,
        },
    ));
    let mut rx = subscribe_event(&event_manager, expected_event.clone()).await?;

    handle_accept_request(
        repo,
        event_manager.clone(),
        "user-2".to_string(),
        "user-1".to_string(),
    )
    .await
    .unwrap();

    let received = expect_event(&mut rx).await?;
    assert_eq!(expected_event, received);

    Ok(())
}

#[tokio::test]
async fn test_sends_friend_request_answered_event_on_reject() -> anyhow::Result<()> {
    let repo = Arc::new(InMemoryUserRepository::new(sample_users()));
    repo.insert_friend_request(&crate::domain::models::friend_request::FriendRequest {
        from_user_id: "user-1".to_string(),
        to_user_id: "user-2".to_string(),
        created_at: None,
        state: crate::domain::models::friend_request::FriendRequestState::Pending,
    })
    .await
    .unwrap();

    let event_manager = new_event_manager().await?;
    let expected_event = Event::UserEvent(UserEvent::FriendRequestAnsweredEvent(
        FriendRequestAnswered {
            from_username: "alice".to_string(),
            accepted: false,
        },
    ));
    let mut rx = subscribe_event(&event_manager, expected_event.clone()).await?;

    handle_reject_request(
        repo,
        event_manager.clone(),
        "user-2".to_string(),
        "user-1".to_string(),
    )
    .await
    .unwrap();

    let received = expect_event(&mut rx).await?;
    assert_eq!(expected_event, received);

    Ok(())
}
