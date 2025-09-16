use devcord_sqlx_utils::error::Error;
use sqlx::PgPool;

use crate::api_utils::{
    structs::{FriendRange, FriendRequest, FriendRequestRange, User},
    types::UserID,
};

//--------------------GETTERS--------------------

pub async fn get_user(user_id: &UserID, db: &PgPool) -> Option<User> {
    sqlx::query_as(
        "
        SELECT username, id, created_at 
        FROM users
        WHERE id = $1
    ",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .ok()
}

pub async fn get_user_block(user_id: &UserID, blocked_id: &UserID, db: &PgPool) -> Option<User> {
    sqlx::query_as(
        "
        SELECT username, id, created_at
        FROM users u
        JOIN blocks b
            ON b.from_user_id = $1
            AND b.to_user_id = u.id
        WHERE u.id = $2
    ",
    )
    .bind(user_id)
    .bind(blocked_id)
    .fetch_one(db)
    .await
    .ok()
}

pub async fn get_user_friend(user_id: &UserID, friend_id: &UserID, db: &PgPool) -> Option<User> {
    sqlx::query_as(
        "
        SELECT username, id, created_at
        FROM users u
        JOIN friendships f
            ON f.from_user_id = $1
            AND f.to_user_id = u.id
        WHERE u.id = $2
    ",
    )
    .bind(user_id)
    .bind(friend_id)
    .fetch_one(db)
    .await
    .ok()
}

pub async fn get_friend_request(request: &FriendRequest, db: &PgPool) -> Option<FriendRequest> {
    sqlx::query_as(
        "
        SELECT from_user_id, to_user_id, created_at
        FROM friend_requests
        WHERE from_user_id = $1 AND to_user_id = $2
    ",
    )
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .fetch_one(db)
    .await
    .ok()
}

pub async fn get_friend_requests_sent(
    user_id: &UserID,
    range: &FriendRequestRange,
    db: &PgPool,
) -> Option<Vec<FriendRequest>> {
    let mut query = String::from(
        "
        SELECT from_user_id, to_user_id, created_at, state
        FROM friend_requests
        WHERE from_user_id = $1
    ",
    );

    if range.state_filter.is_some() {
        query.push_str(" AND state = $4");
    }

    query.push_str(
        "
        ORDER BY created_at DESC
        OFFSET $2
        Limit $3
    ",
    );
    let mut q = sqlx::query_as(&query)
        .bind(user_id)
        .bind(&range.from)
        .bind((&range.to - &range.from).max(0));

    if let Some(filter) = &range.state_filter {
        q = q.bind(filter);
    }

    q.fetch_all(db).await.ok()
}

pub async fn get_friend_requests_received(
    user_id: &UserID,
    range: &FriendRequestRange,
    db: &PgPool,
) -> Option<Vec<FriendRequest>> {
    let mut query = String::from(
        "
        SELECT from_user_id, to_user_id, created_at, state
        FROM friend_requests
        WHERE to_user_id = $1
    ",
    );

    if range.state_filter.is_some() {
        query.push_str(" AND state = $4");
    }

    query.push_str(
        "
        ORDER BY created_at DESC
        OFFSET $2
        Limit $3
    ",
    );
    let mut q = sqlx::query_as(&query)
        .bind(user_id)
        .bind(&range.from)
        .bind((&range.to - &range.from).max(0));

    if let Some(filter) = &range.state_filter {
        q = q.bind(filter);
    }

    q.fetch_all(db).await.ok()
}

pub async fn get_friends(user_id: &UserID, range: &FriendRange, db: &PgPool) -> Option<Vec<User>> {
    let mut query = String::from(
        "
        SELECT id, username, created_at
        FROM users u
        JOIN friendships f
        ON f.to_user_id = u.id
        WHERE f.from_user_id = $1
    ",
    );

    if range.starts_with.is_some() {
        query.push_str(" AND u.username ILIKE $4");
    }

    query.push_str(
        "
        ORDER BY f.created_at DESC
        OFFSET $2
        Limit $3
    ",
    );

    let mut q = sqlx::query_as(&query)
        .bind(user_id)
        .bind(&range.from)
        .bind((&range.to - &range.from).max(0));

    if let Some(filter) = &range.starts_with {
        q = q.bind(filter);
    }

    q.fetch_all(db).await.ok()
}

//--------------------INSERTS--------------------

pub async fn insert_friend_request(request: &FriendRequest, db: &PgPool) -> Result<(), Error> {
    sqlx::query(
        "
        INSERT
        INTO friend_requests (from_user_id, to_user_id)
        VALUES ($1, $2)
    ",
    )
    .bind(&request.from_user_id)
    .bind(&request.to_user_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn insert_friendship(user_a: &UserID, user_b: &UserID, db: &PgPool) -> Result<(), Error> {
    sqlx::query(
        "
        INSERT
        INTO friendships (from_user_id, to_user_id)
        VALUES ($1, $2), ($2, $1)
    ",
    )
    .bind(user_a)
    .bind(user_b)
    .execute(db)
    .await?;

    Ok(())
}

//--------------------DELETE--------------------

//--------------------UPDATE--------------------

pub async fn update_friend_request(request: &FriendRequest, db: &PgPool) -> Result<(), Error> {
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
    .execute(db)
    .await?;

    Ok(())
}
