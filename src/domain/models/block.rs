use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::types::{Time, UserID};

#[derive(FromRow, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Block {
    #[serde(skip_deserializing, rename = "sender_id")]
    pub from_user_id: UserID,
    #[serde(rename(deserialize = "user_id", serialize = "recipient_id"))]
    pub to_user_id: UserID,
    #[serde(skip_deserializing)]
    pub created_at: Option<Time>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.from_user_id == other.from_user_id && self.to_user_id == other.to_user_id
    }
}
