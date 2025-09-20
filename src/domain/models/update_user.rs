use serde::{Deserialize, Serialize};

use crate::domain::types::Username;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct UpdateUser {
    #[serde(default)]
    pub username: Option<Username>,
}

impl UpdateUser {
    pub fn is_empty(&self) -> bool {
        self.username.is_none()
    }
}
