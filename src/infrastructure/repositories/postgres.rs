use sqlx::{Pool, Postgres, QueryBuilder};

use crate::{
    application::repositories::user_repository::UserRepository,
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

impl UserRepository for Pool<Postgres> {
    async fn get_user(&self, user_id: &UserID) -> Option<User> {
        sqlx::query_as(
            "
        SELECT username, id, created_at 
        FROM users
        WHERE id = $1
    ",
        )
        .bind(user_id)
        .fetch_one(self)
        .await
        .ok()
    }

    async fn get_user_friend(&self, user_id: &UserID, friend_id: &UserID) -> Option<User> {
        sqlx::query_as(
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
        .fetch_one(self)
        .await
        .ok()
    }

    async fn get_friend_request(&self, request: &FriendRequest) -> Option<FriendRequest> {
        sqlx::query_as(
            "
        SELECT from_user_id, to_user_id, created_at, state
        FROM friend_requests
        WHERE from_user_id = $1 AND to_user_id = $2
    ",
        )
        .bind(&request.from_user_id)
        .bind(&request.to_user_id)
        .fetch_one(self)
        .await
        .ok()
    }

    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Option<Vec<FriendRequest>> {
        let mut qb = QueryBuilder::new(
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
            qb.push(" AND state = ").push_bind(format!("{}%", filter));
        }

        qb.push(
            " 
        ORDER BY created_at DESC
        OFFSET 
    ",
        )
        .push_bind(range.from)
        .push(" LIMIT ")
        .push_bind((range.to - range.from).max(0))
        .build_query_as()
        .fetch_all(self)
        .await
        .ok()
    }

    async fn get_user_friends(&self, user_id: &UserID, range: &Range) -> Option<Vec<User>> {
        sqlx::query_as(
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
        .fetch_all(self)
        .await
        .ok()
    }

    async fn get_user_block(&self, user_id: &UserID, blocked_id: &UserID) -> Option<User> {
        sqlx::query_as(
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
        .fetch_one(self)
        .await
        .ok()
    }

    async fn get_user_blocks(&self, user_id: &UserID, range: &Range) -> Option<Vec<Block>> {
        sqlx::query_as(
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
        .fetch_all(self)
        .await
        .ok()
    }

    async fn insert_friend_request(
        &self,
        request: &FriendRequest,
    ) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
        INSERT
        INTO friend_requests (from_user_id, to_user_id)
        VALUES ($1, $2)
    ",
        )
        .bind(&request.from_user_id)
        .bind(&request.to_user_id)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn insert_friendship(
        &self,
        user_a: &UserID,
        user_b: &UserID,
    ) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
        INSERT
        INTO friendships (from_user_id, to_user_id)
        VALUES ($1, $2), ($2, $1)
    ",
        )
        .bind(user_a)
        .bind(user_b)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn insert_block(&self, request: &Block) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
        INSERT
        INTO blocks (from_user_id, to_user_id)
        VALUES ($1, $2)
    ",
        )
        .bind(&request.from_user_id)
        .bind(&request.to_user_id)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn insert_user(&self, user: &User) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
            INSERT
            INTO users (username, id)
            VALUES ($1, $2)
        ",
        )
        .bind(&user.username)
        .bind(&user.id)
        .execute(self)
        .await?;

        //FIXME!(Lamoara) make it so the created_at is inserted if is some

        Ok(())
    }

    async fn update_friend_request(
        &self,
        request: &FriendRequest,
    ) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
        UPDATE friend_requests
        SET state = $1, responded_at = CURRENT_TIMESTAMP,
        WHERE from_user_id = $2 AND to_user_id = $3
    ",
        )
        .bind(&request.state)
        .bind(&request.from_user_id)
        .bind(&request.to_user_id)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn update_user(
        &self,
        user_id: &UserID,
        update: &UpdateUser,
    ) -> Result<(), devcord_sqlx_utils::error::Error> {
        if update.is_empty() {
            return Ok(());
        }

        let mut first = true;

        let mut qb = QueryBuilder::new(
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

        qb.build().execute(self).await?;

        Ok(())
    }

    async fn delete_block(&self, block: &Block) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
            DELETE
            FROM blocks
            WHERE from_user_id = $1 AND to_user_id = $2
        ",
        )
        .bind(&block.from_user_id)
        .bind(&block.to_user_id)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn delete_friendship(
        &self,
        friendship: &Friendship,
    ) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
            DELETE
            FROM friendships
            WHERE (from_user_id = $1 AND to_user_id = $2) OR (from_user_id = $2 AND to_user_id = $1)
        ",
        )
        .bind(&friendship.from_user_id)
        .bind(&friendship.to_user_id)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn delete_friend_request(
        &self,
        request: &FriendRequest,
    ) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
            DELETE
            FROM friend_requests
            WHERE from_user_id = $1 AND to_user_id = $2 AND state = $3
        ",
        )
        .bind(&request.from_user_id)
        .bind(&request.to_user_id)
        .bind(&request.state)
        .execute(self)
        .await?;

        Ok(())
    }

    async fn delete_user(&self, user: &User) -> Result<(), devcord_sqlx_utils::error::Error> {
        sqlx::query(
            "
            DELETE
            FROM users
            WHERE id = $1
        ",
        )
        .bind(&user.id)
        .execute(self)
        .await?;

        Ok(())
    }
}
