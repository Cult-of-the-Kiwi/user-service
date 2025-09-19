mod postgres;

use crate::{
    api_utils::structs::{Block, FriendRequest, Friendship, Range, UpdateUser, User},
    sql_utils::calls::UserRepository,
};
use devcord_sqlx_utils::error::Error;

// ---------- HELPERS ----------
async fn create_user<T: UserRepository>(db: &T, id: &str, name: &str) -> User {
    let user = User {
        username: name.to_owned(),
        id: id.to_owned(),
        created_at: None,
    };
    assert!(db.insert_user(&user).await.is_ok());
    user
}

async fn create_friend_request<T: UserRepository>(db: &T, from: &User, to: &User) -> FriendRequest {
    let req = FriendRequest {
        from_user_id: from.id.clone(),
        to_user_id: to.id.clone(),
        created_at: None,
        state: Default::default(),
    };
    assert!(db.insert_friend_request(&req).await.is_ok());
    req
}

async fn create_friendship<T: UserRepository>(db: &T, a: &User, b: &User) -> Friendship {
    let fs = Friendship {
        from_user_id: a.id.clone(),
        to_user_id: b.id.clone(),
        created_at: None,
    };
    assert!(db.insert_friendship(&a.id, &b.id).await.is_ok());
    fs
}

async fn create_block<T: UserRepository>(db: &T, from: &User, to: &User) -> Block {
    let block = Block {
        from_user_id: from.id.clone(),
        to_user_id: to.id.clone(),
        created_at: None,
    };
    assert!(db.insert_block(&block).await.is_ok());
    block
}

// ---------- USER TESTS ----------
pub async fn insert_user_ok<T: UserRepository>(db: &T) {
    let u = create_user(db, "user-1", "Alice").await;
    let fetched = db.get_user(&u.id).await;
    assert_eq!(Some(u), fetched);
}

pub async fn insert_user_duplicate_id<T: UserRepository>(db: &T) {
    let _ = create_user(db, "user-1", "Alice").await;
    let dup = User {
        id: "user-1".into(),
        username: "Bob".into(),
        created_at: None,
    };
    assert_eq!(db.insert_user(&dup).await, Err(Error::AlreadyExists));
}

pub async fn insert_user_duplicate_name<T: UserRepository>(db: &T) {
    let _ = create_user(db, "user-a", "Alice").await;
    let dup = User {
        id: "user-b".into(),
        username: "Alice".into(),
        created_at: None,
    };
    assert_eq!(db.insert_user(&dup).await, Err(Error::AlreadyExists));
}

pub async fn update_user_name<T: UserRepository>(db: &T) {
    let mut u = create_user(db, "user-x", "OldName").await;
    let upd = UpdateUser {
        username: Some("NewName".into()),
    };
    assert!(db.update_user(&u.id, &upd).await.is_ok());
    u.username = "NewName".into();
    assert_eq!(db.get_user(&u.id).await, Some(u));
}

pub async fn update_user_name_without_changing_others<T: UserRepository>(db: &T) {
    let mut u = create_user(db, "user-x", "OldName").await;
    let other_u = create_user(db, "x", "x").await;
    let upd = UpdateUser {
        username: Some("NewName".into()),
    };
    assert!(db.update_user(&u.id, &upd).await.is_ok());
    u.username = "NewName".into();
    assert_eq!(db.get_user(&u.id).await, Some(u));
    assert_eq!(db.get_user(&other_u.id).await, Some(other_u));
}

pub async fn delete_user_ok<T: UserRepository>(db: &T) {
    let u = create_user(db, "user-del", "Temp").await;
    assert!(db.delete_user(&u).await.is_ok());
    assert_eq!(db.get_user(&u.id).await, None);
}

// ---------- FRIEND REQUEST TESTS ----------
pub async fn insert_friend_request_ok<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let req = create_friend_request(db, &a, &b).await;
    assert_eq!(db.get_friend_request(&req).await, Some(req));
}

pub async fn insert_friend_request_duplicate<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let req = create_friend_request(db, &a, &b).await;
    assert_eq!(
        db.insert_friend_request(&req).await,
        Err(Error::AlreadyExists)
    );
}

pub async fn delete_friend_request_ok<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let req = create_friend_request(db, &a, &b).await;
    assert!(db.delete_friend_request(&req).await.is_ok());
    assert_eq!(db.get_friend_request(&req).await, None);
}

// ---------- FRIENDSHIP TESTS ----------
pub async fn insert_friendship_ok<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    create_friendship(db, &a, &b).await;
    let f = db.get_user_friend(&a.id, &b.id).await;
    assert_eq!(Some(b), f);
}

pub async fn delete_friendship_ok<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let fs = create_friendship(db, &a, &b).await;
    assert!(db.delete_friendship(&fs).await.is_ok());
    assert_eq!(db.get_user_friend(&a.id, &b.id).await, None);
}

pub async fn get_friendships<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let c = create_user(db, "user-c", "Carl").await;
    let d = create_user(db, "user-d", "Dominic").await;
    let e = create_user(db, "user-e", "Elisabeth").await;
    create_friendship(db, &a, &b).await;
    create_friendship(db, &a, &c).await;
    create_friendship(db, &a, &d).await;
    create_friendship(db, &a, &e).await;
    let range = Range { from: 0, to: 5 };
    let friend_list = db.get_user_friends(&a.id, &range).await;
    assert_eq!(Some(vec![b, c, d, e]), friend_list);
}

// ---------- BLOCK TESTS ----------
pub async fn insert_block_ok<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let block = create_block(db, &a, &b).await;
    let blocks = db
        .get_user_blocks(&a.id, &Range { from: 0, to: 10 })
        .await
        .unwrap();
    assert!(blocks.contains(&block));
}

pub async fn delete_block_ok<T: UserRepository>(db: &T) {
    let a = create_user(db, "user-a", "Alice").await;
    let b = create_user(db, "user-b", "Bob").await;
    let block = create_block(db, &a, &b).await;
    assert!(db.delete_block(&block).await.is_ok());
    let blocks = db
        .get_user_blocks(&a.id, &Range { from: 0, to: 10 })
        .await
        .unwrap();
    assert!(!blocks.contains(&block));
}
