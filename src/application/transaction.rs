use async_trait::async_trait;
use devcord_sqlx_utils::error::Error;

#[async_trait]
pub(crate) trait Transaction {
    async fn commit(self: Box<Self>) -> Result<(), Error>;
    async fn rollback(self: Box<Self>) -> Result<(), Error>;
}
