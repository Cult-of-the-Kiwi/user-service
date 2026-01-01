use std::{sync::Arc, usize};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use devcord_events::{events::Event, publisher::EventManager};
use devcord_middlewares::models::claims::Claims;
use jsonwebtoken::{EncodingKey, Header};
use serde_json::json;
use tower::ServiceExt;

use crate::{
    api,
    app::AppState,
    application::repositories::user_repository::UserRepository,
    domain::models::{
        friend_request::{FriendRequestDirection, FriendRequestRange, FriendRequestState},
        update_user::UpdateUser,
        user::User,
    },
    tests::{
        in_memory_event_manager::InMemoryEventManager,
        in_memory_user_repository::{InMemoryUserRepository, sample_users},
    },
};

fn jwt() -> String {
    jwt_for("user-1")
}

fn jwt_for(user_id: &str) -> String {
    static SECRET: &str = "test-secret";
    unsafe { std::env::set_var("JWT_SECRET", SECRET) };
    let claims = Claims {
        exp: (chrono::Utc::now() + chrono::Duration::minutes(5)).timestamp() as u64,
        user_id: user_id.to_string(),
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap()
}

fn build_router() -> (
    Router,
    Arc<InMemoryUserRepository>,
    Arc<InMemoryEventManager>,
) {
    let repo = Arc::new(InMemoryUserRepository::new(sample_users()));
    let events = Arc::new(InMemoryEventManager::new());
    let state = Arc::new(AppState {
        db: repo.clone() as Arc<dyn UserRepository>,
        event_manager: events.clone() as Arc<dyn EventManager<Event = Event>>,
    });
    let router = api::new().with_state(state);
    (router, repo, events)
}

#[tokio::test]
async fn test_health_works() {
    let (router, ..) = build_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload, json!({ "status": "ok" }));
}

#[tokio::test]
async fn test_get_user_returns_user() {
    let (router, ..) = build_router();
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/user-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user: User = serde_json::from_slice(&body).unwrap();
    assert_eq!(user.username, "alice");
}

#[tokio::test]
async fn test_update_user_updates_name() {
    let (router, repo, ..) = build_router();
    let token = jwt();

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/update")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::from(
                    serde_json::to_vec(&UpdateUser {
                        username: Some("alice-updated".to_string()),
                    })
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated = repo.get_user(&"user-1".to_string()).await.unwrap();
    assert_eq!(updated.username, "alice-updated");
}

#[tokio::test]
async fn test_friendship_request_and_accept_flow() {
    let (router, repo, ..) = build_router();
    let requester = jwt_for("user-1");

    let request_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/requests/user-2")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    let accept_token = jwt_for("user-2");

    let accept_response = router
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/friendship/requests/user-1/accept")
                .header(header::AUTHORIZATION, format!("Bearer {accept_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(accept_response.status(), StatusCode::OK);

    let friendships = repo
        .get_friend_requests(
            &"user-1".to_string(),
            &FriendRequestRange {
                from: 0,
                to: 10,
                state_filter: Some(FriendRequestState::Accepted),
            },
            &FriendRequestDirection::Sent,
        )
        .await
        .unwrap();
    assert!(!friendships.is_empty());
}

#[tokio::test]
async fn test_delete_friend_request() {
    let (router, repo, ..) = build_router();
    let requester = jwt_for("user-1");

    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/requests/user-2")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/friendship/requests/user-2")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let remaining = repo
        .get_friend_requests(
            &"user-1".to_string(),
            &FriendRequestRange {
                from: 0,
                to: 10,
                state_filter: None,
            },
            &FriendRequestDirection::Sent,
        )
        .await
        .unwrap();
    assert!(remaining.is_empty());
}

#[tokio::test]
async fn test_is_friend_endpoint() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");
    let acceptor = jwt_for("user-2");

    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/requests/user-2")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/friendship/requests/user-1/accept")
                .header(header::AUTHORIZATION, format!("Bearer {acceptor}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/user-2/is-friend")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let is_friend: bool = serde_json::from_slice(&body).unwrap();
    assert!(is_friend);
}

#[tokio::test]
async fn test_is_friend_endpoint_returns_false_when_not_friend() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/user-2/is-friend")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let is_friend: bool = serde_json::from_slice(&body).unwrap();
    assert!(!is_friend);
}

#[tokio::test]
async fn test_is_blocked_endpoint() {
    let (router, ..) = build_router();
    let blocker = jwt_for("user-1");

    let block_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/blocks/user-2")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(block_response.status(), StatusCode::OK);

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/user-2/is-blocked")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let is_blocked: bool = serde_json::from_slice(&body).unwrap();
    assert!(is_blocked);
}

#[tokio::test]
async fn test_is_blocked_endpoint_returns_false_when_not_blocked() {
    let (router, ..) = build_router();
    let blocker = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/user-2/is-blocked")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let is_blocked: bool = serde_json::from_slice(&body).unwrap();
    assert!(!is_blocked);
}

#[tokio::test]
async fn test_cannot_send_friend_request_to_self() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/requests/user-1")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot send a friend request to yourself");
}

#[tokio::test]
async fn test_cannot_block_self() {
    let (router, ..) = build_router();
    let blocker = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/blocks/user-1")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot block yourself");
}

#[tokio::test]
async fn test_cannot_accept_friend_request_to_self() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/friendship/requests/user-1/accept")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot accept a friend request to yourself");
}

#[tokio::test]
async fn test_cannot_reject_friend_request_to_self() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/friendship/requests/user-1/reject")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot reject a friend request to yourself");
}

#[tokio::test]
async fn test_cannot_delete_friend_request_to_self() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/friendship/requests/user-1")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot delete a friend request to yourself");
}

#[tokio::test]
async fn test_cannot_remove_self_as_friend() {
    let (router, ..) = build_router();
    let requester = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/friendship/friends/user-1")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot remove yourself as a friend");
}

#[tokio::test]
async fn test_cannot_unblock_self() {
    let (router, ..) = build_router();
    let blocker = jwt_for("user-1");

    let response = router
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/blocks/user-1")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "You cannot unblock yourself");
}
