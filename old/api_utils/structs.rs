use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::api_utils::types::{UserID, UserUsername};

#[derive(FromRow, Debug, Default, Deserialize, Serialize)]
pub(crate) struct User {
    pub username: UserUsername,
    pub id: UserID,
    pub created_at: Option<DateTime<Utc>>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(
    Debug, Default, Deserialize, Serialize, sqlx::Type, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[sqlx(type_name = "friend_request_state", rename_all = "lowercase")]
pub enum FriendRequestState {
    #[default]
    Pending,
    Accepted,
    Rejected,
}

impl From<&str> for FriendRequestState {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "pending" => Self::Pending,
            "accepted" => Self::Accepted,
            "rejected" => Self::Rejected,
            _ => Self::default(),
        }
    }
}

impl Display for FriendRequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FriendRequestState::Pending => write!(f, "pending"),
            FriendRequestState::Accepted => write!(f, "accepted"),
            FriendRequestState::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(FromRow, Debug, Default, Deserialize, Serialize)]
pub(crate) struct FriendRequest {
    #[serde(skip_deserializing, rename(serialize = "sender_id"))]
    pub from_user_id: UserID,
    #[serde(rename(deserialize = "user_id", serialize = "recipient_id"))]
    pub to_user_id: UserID,
    #[serde(skip_deserializing)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_deserializing)]
    pub state: FriendRequestState,
}

impl PartialEq for FriendRequest {
    fn eq(&self, other: &Self) -> bool {
        self.from_user_id == other.from_user_id && self.to_user_id == other.to_user_id
    }
}

impl FriendRequest {
    pub fn accept(&mut self) {
        self.state = FriendRequestState::Accepted;
    }

    pub fn reject(&mut self) {
        self.state = FriendRequestState::Rejected;
    }

    pub fn inverted(&self) -> FriendRequest {
        FriendRequest {
            from_user_id: self.to_user_id.clone(),
            to_user_id: self.from_user_id.clone(),
            created_at: self.created_at,
            state: self.state,
        }
    }

    pub fn is_pending(&self) -> bool {
        self.state == FriendRequestState::Pending
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct FriendRequestRange {
    pub from: i32,
    pub to: i32,
    #[serde(default, rename = "filter")]
    pub state_filter: Option<FriendRequestState>,
}

pub(crate) enum FriendRequestDirection {
    Sent,
    Received,
}

#[derive(FromRow, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Friendship {
    #[serde(skip_deserializing, rename = "sender_id")]
    pub from_user_id: UserID,
    #[serde(rename(deserialize = "user_id", serialize = "recipient_id"))]
    pub to_user_id: UserID,
    #[serde(skip_deserializing)]
    pub created_at: Option<DateTime<Utc>>,
}

impl PartialEq for Friendship {
    fn eq(&self, other: &Self) -> bool {
        self.from_user_id == other.from_user_id && self.to_user_id == other.to_user_id
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Range {
    pub from: i32,
    pub to: i32,
}

#[derive(FromRow, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Block {
    #[serde(skip_deserializing, rename = "sender_id")]
    pub from_user_id: UserID,
    #[serde(rename(deserialize = "user_id", serialize = "recipient_id"))]
    pub to_user_id: UserID,
    #[serde(skip_deserializing)]
    pub created_at: Option<DateTime<Utc>>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.from_user_id == other.from_user_id && self.to_user_id == other.to_user_id
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct UpdateUser {
    #[serde(default)]
    pub username: Option<UserUsername>,
}

impl UpdateUser {
    pub fn is_empty(&self) -> bool {
        self.username.is_none()
    }
}
