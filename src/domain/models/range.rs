use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Range {
    pub from: i32,
    pub to: i32,
}
