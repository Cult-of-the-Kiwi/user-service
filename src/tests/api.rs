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

fn json_request(method: Method, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
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
    assert_eq!(body, "Long life to the allmighty turbofish");
}

#[tokio::test]
async fn test_get_user_returns_user() {
    let (router, ..) = build_router();
    let response = router
        .oneshot(json_request(
            Method::GET,
            "/",
            json!({ "id": "user-1", "username": "" }),
        ))
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

    // user-1 -> user-2 request
    let request_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/request")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);

    // prepare accept token for user-2
    let accept_token = jwt_for("user-2");

    let accept_response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/accept")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {accept_token}"))
                .body(Body::from(json!({ "user_id": "user-1" }).to_string()))
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

    // user-1 -> user-2 request
    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/request")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // delete request
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/delete")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
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

    // create friend request
    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/request")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // accept it
    let _ = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/accept")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {acceptor}"))
                .body(Body::from(json!({ "user_id": "user-1" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // check friend
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/friendship/is_friend")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::from(
                    json!({ "id": "user-2", "username": "" }).to_string(),
                ))
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
                .method(Method::POST)
                .uri("/friendship/is_friend")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {requester}"))
                .body(Body::from(
                    json!({ "id": "user-2", "username": "" }).to_string(),
                ))
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

    // block user-2
    let block_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/blocks/block")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(block_response.status(), StatusCode::OK);

    // check blocked
    let response = router
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/blocks/is_blocked")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
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
                .method(Method::POST)
                .uri("/blocks/is_blocked")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {blocker}"))
                .body(Body::from(json!({ "user_id": "user-2" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let is_blocked: bool = serde_json::from_slice(&body).unwrap();
    assert!(!is_blocked);
}
