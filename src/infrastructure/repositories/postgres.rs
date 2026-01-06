use async_trait::async_trait;
use devcord_sqlx_utils::error::Error;
use sqlx::{
    Error as SqlxError, Pool, Postgres, QueryBuilder,
    postgres::{PgConnection, PgExecutor},
};
use tokio::sync::Mutex;

use crate::{
    application::{
        repositories::user_repository::{UserRepository, UserRepositoryTx},
        transaction::Transaction,
    },
    domain::{
        models::{
            block::Block,
            friend_request::{FriendRequest, FriendRequestDirection, FriendRequestRange},
            friendship::Friendship,
            range::Range,
            update_user::UpdateUser,
            user::User,
        },
        types::UserID,
    },
};

pub(crate) struct PostgresUserRepository(pub Pool<Postgres>);

impl PostgresUserRepository {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self(pool)
    }
}

pub(crate) struct PostgresUserRepositoryTx<'a> {
    tx: Mutex<Option<sqlx::Transaction<'a, Postgres>>>,
}

impl<'a> PostgresUserRepositoryTx<'a> {
    fn conn<'b>(
        tx: &'b mut Option<sqlx::Transaction<'a, Postgres>>,
    ) -> Result<&'b mut PgConnection, Error> {
        let tx = tx
            .as_mut()
            .ok_or_else(|| SqlxError::Protocol("Transaction already completed".into()))?;
        Ok(&mut **tx)
    }
}

async fn query_get_user<'e, E>(executor: E, user_id: &UserID) -> Result<User, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, User>(
        "
        SELECT username, id, created_at 
        FROM users
        WHERE id = $1
    ",
    )
    .bind(user_id)
    .fetch_one(executor)
    .await
    .map_err(Into::into)
}

async fn query_get_user_if_friend<'e, E>(
    executor: E,
    user_id: &UserID,
    friend_id: &UserID,
) -> Result<User, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, User>(
        "
        SELECT u.username, u.id, u.created_at
        FROM users u
        JOIN friendships f
            ON f.to_user_id = u.id
        WHERE f.from_user_id = $1 AND u.id = $2 
    ",
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(executor)
    .await
    .map_err(Into::into)
}

async fn query_get_friend_request<'e, E>(
    executor: E,
    request: &FriendRequest,
) -> Result<FriendRequest, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, FriendRequest>(
        "
        SELECT from_user_id, to_user_id, created_at, state
        FROM friend_requests
        WHERE from_user_id = $1 AND to_user_id = $2
    ",
    )
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .fetch_one(executor)
    .await
    .map_err(Into::into)
}

async fn query_get_friend_requests<'e, E>(
    executor: E,
    user_id: &UserID,
    range: &FriendRequestRange,
    direction: &FriendRequestDirection,
) -> Result<Vec<FriendRequest>, Error>
where
    E: PgExecutor<'e>,
{
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "
        SELECT from_user_id, to_user_id, created_at, state
        FROM friend_requests
        WHERE 
    ",
    );

    match direction {
        FriendRequestDirection::Sent => qb.push("from_user_id = "),
        FriendRequestDirection::Received => qb.push("to_user_id = "),
    };

    qb.push_bind(user_id);

    if let Some(filter) = &range.state_filter {
        qb.push(" AND state = ").push_bind(filter);
    }

    qb.push(
        " 
        ORDER BY created_at DESC
        OFFSET 
    ",
    )
    .push_bind(range.from)
    .push(" LIMIT ")
    .push_bind((range.to - range.from).max(0));

    qb.build_query_as::<FriendRequest>()
        .fetch_all(executor)
        .await
        .map_err(Into::into)
}

async fn query_get_user_friends<'e, E>(
    executor: E,
    user_id: &UserID,
    range: &Range,
) -> Result<Vec<User>, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, User>(
        "
        SELECT u.id, u.username, u.created_at
        FROM users u
        JOIN friendships f
        ON f.to_user_id = u.id
        WHERE f.from_user_id = $1
        ORDER BY f.created_at ASC
        OFFSET $2
        LIMIT $3
    ",
    )
    .bind(user_id)
    .bind(range.from)
    .bind((range.to - range.from).max(0))
    .fetch_all(executor)
    .await
    .map_err(Into::into)
}

async fn query_get_user_if_blocked<'e, E>(
    executor: E,
    user_id: &UserID,
    blocked_id: &UserID,
) -> Result<User, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, User>(
        "
        SELECT username, id, created_at
        FROM users u
        JOIN blocks b
            ON b.to_user_id = u.id
        WHERE b.from_user_id = $1 AND u.id = $2
    ",
    )
    .bind(user_id)
    .bind(blocked_id)
    .fetch_one(executor)
    .await
    .map_err(Into::into)
}

async fn query_get_user_blocks<'e, E>(
    executor: E,
    user_id: &UserID,
    range: &Range,
) -> Result<Vec<Block>, Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as::<_, Block>(
        "
            SELECT from_user_id, to_user_id, created_at
            FROM blocks
            WHERE from_user_id = $1
            ORDER BY created_at DESC
            OFFSET $2
            LIMIT $3
        ",
    )
    .bind(user_id)
    .bind(range.from)
    .bind((range.to - range.from).max(0))
    .fetch_all(executor)
    .await
    .map_err(Into::into)
}

async fn query_insert_friend_request<'e, E>(
    executor: E,
    request: &FriendRequest,
) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
        INSERT
        INTO friend_requests (from_user_id, to_user_id)
        VALUES ($1, $2)
    ",
    )
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_insert_friendship<'e, E>(
    executor: E,
    user_a: &UserID,
    user_b: &UserID,
) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
        INSERT
        INTO friendships (from_user_id, to_user_id)
        VALUES ($1, $2), ($2, $1)
        ",
    )
    .bind(user_a)
    .bind(user_b)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_insert_block<'e, E>(executor: E, request: &Block) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
        INSERT
        INTO blocks (from_user_id, to_user_id)
        VALUES ($1, $2)
        ",
    )
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_insert_user<'e, E>(executor: E, user: &User) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "
            INSERT INTO users (username, id",
    );

    if user.created_at.is_some() {
        qb.push(", created_at");
    }

    qb.push(") VALUES (");
    qb.push_bind(&user.username);
    qb.push(", ").push_bind(&user.id);

    if let Some(created_at) = user.created_at {
        qb.push(", ").push_bind(created_at);
    }

    qb.push(")");

    qb.build()
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
}

async fn query_update_friend_request<'e, E>(
    executor: E,
    request: &FriendRequest,
) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
        UPDATE friend_requests
        SET state = $1, responded_at = CURRENT_TIMESTAMP,
        WHERE from_user_id = $2 AND to_user_id = $3
        ",
    )
    .bind(request.state)
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_update_user<'e, E>(
    executor: E,
    user_id: &UserID,
    update: &UpdateUser,
) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    if update.is_empty() {
        return Ok(());
    }

    let mut first = true;

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "
            UPDATE users
            SET
        ",
    );

    if let Some(username) = &update.username {
        if !first {
            qb.push(", ");
        }
        qb.push("username = ").push_bind(username);
        first = false;
    }

    qb.push(" WHERE id =  ");
    qb.push_bind(user_id);

    qb.build()
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
}

async fn query_delete_block<'e, E>(executor: E, block: &Block) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
            DELETE
            FROM blocks
            WHERE from_user_id = $1 AND to_user_id = $2
        ",
    )
    .bind(&block.from_user_id)
    .bind(&block.to_user_id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_delete_friendship<'e, E>(executor: E, friendship: &Friendship) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
            DELETE
            FROM friendships
            WHERE (from_user_id = $1 AND to_user_id = $2) OR (from_user_id = $2 AND to_user_id = $1)
        ",
    )
    .bind(&friendship.from_user_id)
    .bind(&friendship.to_user_id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_delete_friend_request<'e, E>(
    executor: E,
    request: &FriendRequest,
) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
            DELETE
            FROM friend_requests
            WHERE from_user_id = $1 AND to_user_id = $2 AND state = $3
        ",
    )
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .bind(request.state)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

async fn query_delete_user<'e, E>(executor: E, user: &User) -> Result<(), Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "
            DELETE
            FROM users
            WHERE id = $1
        ",
    )
    .bind(&user.id)
    .execute(executor)
    .await
    .map(|_| ())
    .map_err(Into::into)
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn begin_tx(&self) -> Result<Box<dyn UserRepositoryTx + '_>, Error> {
        let tx = self.0.begin().await?;
        Ok(Box::new(PostgresUserRepositoryTx {
            tx: Mutex::new(Some(tx)),
        }))
    }

    async fn get_user(&self, user_id: &UserID) -> Result<User, Error> {
        query_get_user(&self.0, user_id).await
    }

    async fn get_user_if_friend(
        &self,
        user_id: &UserID,
        friend_id: &UserID,
    ) -> Result<User, Error> {
        query_get_user_if_friend(&self.0, user_id, friend_id).await
    }

    async fn get_friend_request(&self, request: &FriendRequest) -> Result<FriendRequest, Error> {
        query_get_friend_request(&self.0, request).await
    }

    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Result<Vec<FriendRequest>, Error> {
        query_get_friend_requests(&self.0, user_id, range, direction).await
    }

    async fn get_user_friends(&self, user_id: &UserID, range: &Range) -> Result<Vec<User>, Error> {
        query_get_user_friends(&self.0, user_id, range).await
    }

    async fn get_user_if_blocked(
        &self,
        user_id: &UserID,
        blocked_id: &UserID,
    ) -> Result<User, Error> {
        query_get_user_if_blocked(&self.0, user_id, blocked_id).await
    }

    async fn get_user_blocks(&self, user_id: &UserID, range: &Range) -> Result<Vec<Block>, Error> {
        query_get_user_blocks(&self.0, user_id, range).await
    }

    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), Error> {
        query_insert_friend_request(&self.0, request).await
    }

    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), Error> {
        query_insert_friendship(&self.0, user_a, user_b).await
    }

    async fn insert_block(&self, request: &Block) -> Result<(), Error> {
        query_insert_block(&self.0, request).await
    }

    async fn insert_user(&self, user: &User) -> Result<(), Error> {
        query_insert_user(&self.0, user).await
    }

    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), Error> {
        query_update_friend_request(&self.0, request).await
    }

    async fn update_user(&self, user_id: &UserID, update: &UpdateUser) -> Result<(), Error> {
        query_update_user(&self.0, user_id, update).await
    }

    async fn delete_block(&self, block: &Block) -> Result<(), Error> {
        query_delete_block(&self.0, block).await
    }

    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), Error> {
        query_delete_friendship(&self.0, friendship).await
    }

    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), Error> {
        query_delete_friend_request(&self.0, request).await
    }

    async fn delete_user(&self, user: &User) -> Result<(), Error> {
        query_delete_user(&self.0, user).await
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepositoryTx<'_> {
    async fn begin_tx(&self) -> Result<Box<dyn UserRepositoryTx + '_>, Error> {
        Err(SqlxError::Protocol("Nested transactions are not supported".into()).into())
    }

    async fn get_user(&self, user_id: &UserID) -> Result<User, Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_get_user(conn, user_id).await
    }

    async fn get_user_if_friend(
        &self,
        user_id: &UserID,
        friend_id: &UserID,
    ) -> Result<User, Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_get_user_if_friend(conn, user_id, friend_id).await
    }

    async fn get_friend_request(&self, request: &FriendRequest) -> Result<FriendRequest, Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_get_friend_request(conn, request).await
    }

    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Result<Vec<FriendRequest>, Error> {
        let mut tx_lock = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx_lock)?;
        query_get_friend_requests(conn, user_id, range, direction).await
    }

    async fn get_user_friends(&self, user_id: &UserID, range: &Range) -> Result<Vec<User>, Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_get_user_friends(conn, user_id, range).await
    }

    async fn get_user_if_blocked(
        &self,
        user_id: &UserID,
        blocked_id: &UserID,
    ) -> Result<User, Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_get_user_if_blocked(conn, user_id, blocked_id).await
    }

    async fn get_user_blocks(&self, user_id: &UserID, range: &Range) -> Result<Vec<Block>, Error> {
        let mut tx_lock = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx_lock)?;
        query_get_user_blocks(conn, user_id, range).await
    }

    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_insert_friend_request(conn, request).await
    }

    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_insert_friendship(conn, user_a, user_b).await
    }

    async fn insert_block(&self, request: &Block) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_insert_block(conn, request).await
    }

    async fn insert_user(&self, user: &User) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_insert_user(conn, user).await
    }

    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_update_friend_request(conn, request).await
    }

    async fn update_user(&self, user_id: &UserID, update: &UpdateUser) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_update_user(conn, user_id, update).await
    }

    async fn delete_block(&self, block: &Block) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_delete_block(conn, block).await
    }

    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_delete_friendship(conn, friendship).await
    }

    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_delete_friend_request(conn, request).await
    }

    async fn delete_user(&self, user: &User) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        let conn = PostgresUserRepositoryTx::conn(&mut tx)?;
        query_delete_user(conn, user).await
    }
}

#[async_trait]
impl Transaction for PostgresUserRepositoryTx<'_> {
    async fn commit(self: Box<Self>) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        if let Some(tx) = tx.take() {
            tx.commit().await?;
        }
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), Error> {
        let mut tx = self.tx.lock().await;
        if let Some(tx) = tx.take() {
            tx.rollback().await?;
        }
        Ok(())
    }
}
