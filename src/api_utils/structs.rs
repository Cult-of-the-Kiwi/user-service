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

#[derive(Debug, Default, Deserialize, Serialize, sqlx::Type)]
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

impl FriendRequest {
    pub fn accept(&mut self) {
        self.state = FriendRequestState::Accepted;
    }

    pub fn reject(&mut self) {
        self.state = FriendRequestState::Rejected;
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
