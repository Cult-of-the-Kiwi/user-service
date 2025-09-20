use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::domain::types::{Time, UserID, Username};

#[derive(FromRow, Debug, Default, Deserialize, Serialize)]
pub(crate) struct User {
    pub username: Username,
    pub id: UserID,
    pub created_at: Option<Time>,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
