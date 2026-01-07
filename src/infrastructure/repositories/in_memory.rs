use std::collections::{HashMap, hash_map::Entry::Occupied};

use async_trait::async_trait;
use devcord_sqlx_utils::error::Error as DbError;
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

#[derive(Default)]
struct InMemoryState {
    users: Mutex<HashMap<UserID, User>>,
    friend_requests: Mutex<HashMap<(UserID, UserID), FriendRequest>>,
    friendships: Mutex<Vec<Friendship>>,
    blocks: Mutex<Vec<Block>>,
}

impl InMemoryState {
    fn new(users: Vec<User>) -> Self {
        Self {
            users: Mutex::new(users.into_iter().map(|u| (u.id.clone(), u)).collect()),
            friend_requests: Mutex::new(HashMap::new()),
            friendships: Mutex::new(Vec::new()),
            blocks: Mutex::new(Vec::new()),
        }
    }

    async fn snapshot(&self) -> Self {
        let users = self.users.lock().await.clone();
        let friend_requests = self.friend_requests.lock().await.clone();
        let friendships = self.friendships.lock().await.iter().cloned().collect();
        let blocks = self.blocks.lock().await.clone();

        Self {
            users: Mutex::new(users),
            friend_requests: Mutex::new(friend_requests),
            friendships: Mutex::new(friendships),
            blocks: Mutex::new(blocks),
        }
    }

    async fn replace_with(&self, other: &InMemoryState) {
        *self.users.lock().await = other.users.lock().await.clone();
        *self.friend_requests.lock().await = other.friend_requests.lock().await.clone();
        *self.friendships.lock().await = other.friendships.lock().await.clone();
        *self.blocks.lock().await = other.blocks.lock().await.clone();
    }

    async fn get_user(&self, user_id: &UserID) -> Result<User, DbError> {
        self.users
            .lock()
            .await
            .get(user_id)
            .cloned()
            .ok_or(DbError::RowNotFound)
    }

    async fn get_user_if_friend(
        &self,
        user_id: &UserID,
        friend_id: &UserID,
    ) -> Result<User, DbError> {
        let friendships = self.friendships.lock().await;
        let exists = friendships.iter().any(|f| {
            (f.from_user_id == *user_id && f.to_user_id == *friend_id)
                || (f.to_user_id == *user_id && f.from_user_id == *friend_id)
        });
        drop(friendships);
        if exists {
            self.get_user(friend_id).await
        } else {
            Err(DbError::RowNotFound)
        }
    }

    async fn get_friend_request(&self, request: &FriendRequest) -> Result<FriendRequest, DbError> {
        self.friend_requests
            .lock()
            .await
            .get(&(request.from_user_id.clone(), request.to_user_id.clone()))
            .cloned()
            .ok_or(DbError::RowNotFound)
    }

    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Result<Vec<FriendRequest>, DbError> {
        let requests = self.friend_requests.lock().await;
        let mut filtered: Vec<_> = requests
            .values()
            .filter(|req| match direction {
                FriendRequestDirection::Sent => &req.from_user_id == user_id,
                FriendRequestDirection::Received => &req.to_user_id == user_id,
            })
            .cloned()
            .collect();

        if let Some(state) = range.state_filter {
            filtered.retain(|req| req.state == state);
        }

        Ok(filtered)
    }

    async fn get_user_friends(
        &self,
        user_id: &UserID,
        _range: &Range,
    ) -> Result<Vec<User>, DbError> {
        let friendships = self.friendships.lock().await;
        let users = self.users.lock().await;

        let list = friendships
            .iter()
            .filter_map(|f| {
                if &f.from_user_id == user_id {
                    users.get(&f.to_user_id).cloned()
                } else if &f.to_user_id == user_id {
                    users.get(&f.from_user_id).cloned()
                } else {
                    None
                }
            })
            .collect();

        Ok(list)
    }

    async fn get_user_if_blocked(
        &self,
        _user_id: &UserID,
        _blocked_id: &UserID,
    ) -> Result<User, DbError> {
        let blocks = self.blocks.lock().await;
        let exists = blocks
            .iter()
            .any(|b| &b.from_user_id == _user_id && &b.to_user_id == _blocked_id);
        drop(blocks);

        if exists {
            self.get_user(_blocked_id).await
        } else {
            Err(DbError::RowNotFound)
        }
    }

    async fn get_user_blocks(
        &self,
        _user_id: &UserID,
        _range: &Range,
    ) -> Result<Vec<Block>, DbError> {
        let blocks = self.blocks.lock().await;
        Ok(blocks
            .iter()
            .filter(|b| &b.from_user_id == _user_id)
            .cloned()
            .collect())
    }

    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        let users = self.users.lock().await;
        if !users.contains_key(&request.from_user_id) || !users.contains_key(&request.to_user_id) {
            return Err(DbError::RowNotFound);
        }
        drop(users);

        let mut requests = self.friend_requests.lock().await;
        let key = (request.from_user_id.clone(), request.to_user_id.clone());
        if requests.contains_key(&key) {
            return Err(DbError::AlreadyExists);
        }
        requests.insert(key, request.clone());
        Ok(())
    }

    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), DbError> {
        let users = self.users.lock().await;
        if !users.contains_key(user_a) || !users.contains_key(user_b) {
            return Err(DbError::RowNotFound);
        }
        drop(users);

        self.friendships.lock().await.push(Friendship {
            from_user_id: user_a.clone(),
            to_user_id: user_b.clone(),
            created_at: None,
        });
        Ok(())
    }

    async fn insert_block(&self, _block: &Block) -> Result<(), DbError> {
        let users = self.users.lock().await;
        if !users.contains_key(&_block.from_user_id) || !users.contains_key(&_block.to_user_id) {
            return Err(DbError::RowNotFound);
        }
        drop(users);

        let mut blocks = self.blocks.lock().await;
        if blocks
            .iter()
            .any(|b| b.from_user_id == _block.from_user_id && b.to_user_id == _block.to_user_id)
        {
            return Err(DbError::AlreadyExists);
        }
        blocks.push(_block.clone());
        Ok(())
    }

    async fn insert_user(&self, user: &User) -> Result<(), DbError> {
        let mut users = self.users.lock().await;
        if users.contains_key(&user.id) || users.values().any(|u| u.username == user.username) {
            return Err(DbError::AlreadyExists);
        }
        users.insert(user.id.clone(), user.clone());
        Ok(())
    }

    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        let mut requests = self.friend_requests.lock().await;
        let key = (request.from_user_id.clone(), request.to_user_id.clone());
        if let Occupied(mut e) = requests.entry(key) {
            e.insert(request.clone());
            Ok(())
        } else {
            Err(DbError::RowNotFound)
        }
    }

    async fn update_user(&self, user_id: &UserID, request: &UpdateUser) -> Result<(), DbError> {
        let mut users = self.users.lock().await;
        let Some(user) = users.get_mut(user_id) else {
            return Err(DbError::RowNotFound);
        };
        if let Some(username) = &request.username {
            user.username = username.clone();
        }
        Ok(())
    }

    async fn delete_block(&self, _request: &Block) -> Result<(), DbError> {
        let mut blocks = self.blocks.lock().await;
        if let Some(idx) = blocks.iter().position(|b| {
            b.from_user_id == _request.from_user_id && b.to_user_id == _request.to_user_id
        }) {
            blocks.remove(idx);
            Ok(())
        } else {
            Err(DbError::RowNotFound)
        }
    }

    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), DbError> {
        let mut friendships = self.friendships.lock().await;
        if let Some(idx) = friendships.iter().position(|f| {
            f.from_user_id == friendship.from_user_id && f.to_user_id == friendship.to_user_id
        }) {
            friendships.remove(idx);
            Ok(())
        } else {
            Err(DbError::RowNotFound)
        }
    }

    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        let mut requests = self.friend_requests.lock().await;
        let key = (request.from_user_id.clone(), request.to_user_id.clone());
        if requests.remove(&key).is_some() {
            Ok(())
        } else {
            Err(DbError::RowNotFound)
        }
    }

    async fn delete_user(&self, user: &User) -> Result<(), DbError> {
        self.users
            .lock()
            .await
            .remove(&user.id)
            .map(|_| ())
            .ok_or(DbError::RowNotFound)
    }
}

#[derive(Default)]
pub(crate) struct InMemoryUserRepository {
    state: InMemoryState,
}

impl InMemoryUserRepository {
    pub(crate) fn new(users: Vec<User>) -> Self {
        Self {
            state: InMemoryState::new(users),
        }
    }
}

pub(crate) struct InMemoryUserRepositoryTx<'a> {
    parent: &'a InMemoryState,
    state: InMemoryState,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn begin_tx(&self) -> Result<Box<dyn UserRepositoryTx + '_>, DbError> {
        let snapshot = self.state.snapshot().await;
        Ok(Box::new(InMemoryUserRepositoryTx {
            parent: &self.state,
            state: snapshot,
        }))
    }

    async fn get_user(&self, user_id: &UserID) -> Result<User, DbError> {
        self.state.get_user(user_id).await
    }

    async fn get_user_if_friend(
        &self,
        user_id: &UserID,
        friend_id: &UserID,
    ) -> Result<User, DbError> {
        self.state.get_user_if_friend(user_id, friend_id).await
    }

    async fn get_friend_request(&self, request: &FriendRequest) -> Result<FriendRequest, DbError> {
        self.state.get_friend_request(request).await
    }

    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Result<Vec<FriendRequest>, DbError> {
        self.state
            .get_friend_requests(user_id, range, direction)
            .await
    }

    async fn get_user_friends(
        &self,
        user_id: &UserID,
        range: &Range,
    ) -> Result<Vec<User>, DbError> {
        self.state.get_user_friends(user_id, range).await
    }

    async fn get_user_if_blocked(
        &self,
        user_id: &UserID,
        blocked_id: &UserID,
    ) -> Result<User, DbError> {
        self.state.get_user_if_blocked(user_id, blocked_id).await
    }

    async fn get_user_blocks(
        &self,
        user_id: &UserID,
        range: &Range,
    ) -> Result<Vec<Block>, DbError> {
        self.state.get_user_blocks(user_id, range).await
    }

    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        self.state.insert_friend_request(request).await
    }

    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), DbError> {
        self.state.insert_friendship(user_a, user_b).await
    }

    async fn insert_block(&self, block: &Block) -> Result<(), DbError> {
        self.state.insert_block(block).await
    }

    async fn insert_user(&self, user: &User) -> Result<(), DbError> {
        self.state.insert_user(user).await
    }

    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        self.state.update_friend_request(request).await
    }

    async fn update_user(&self, user_id: &UserID, request: &UpdateUser) -> Result<(), DbError> {
        self.state.update_user(user_id, request).await
    }

    async fn delete_block(&self, request: &Block) -> Result<(), DbError> {
        self.state.delete_block(request).await
    }

    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), DbError> {
        self.state.delete_friendship(friendship).await
    }

    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        self.state.delete_friend_request(request).await
    }

    async fn delete_user(&self, user: &User) -> Result<(), DbError> {
        self.state.delete_user(user).await
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepositoryTx<'_> {
    async fn begin_tx(&self) -> Result<Box<dyn UserRepositoryTx + '_>, DbError> {
        let snapshot = self.state.snapshot().await;
        Ok(Box::new(InMemoryUserRepositoryTx {
            parent: &self.state,
            state: snapshot,
        }))
    }

    async fn get_user(&self, user_id: &UserID) -> Result<User, DbError> {
        self.state.get_user(user_id).await
    }

    async fn get_user_if_friend(
        &self,
        user_id: &UserID,
        friend_id: &UserID,
    ) -> Result<User, DbError> {
        self.state.get_user_if_friend(user_id, friend_id).await
    }

    async fn get_friend_request(&self, request: &FriendRequest) -> Result<FriendRequest, DbError> {
        self.state.get_friend_request(request).await
    }

    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Result<Vec<FriendRequest>, DbError> {
        self.state
            .get_friend_requests(user_id, range, direction)
            .await
    }

    async fn get_user_friends(
        &self,
        user_id: &UserID,
        range: &Range,
    ) -> Result<Vec<User>, DbError> {
        self.state.get_user_friends(user_id, range).await
    }

    async fn get_user_if_blocked(
        &self,
        user_id: &UserID,
        blocked_id: &UserID,
    ) -> Result<User, DbError> {
        self.state.get_user_if_blocked(user_id, blocked_id).await
    }

    async fn get_user_blocks(
        &self,
        user_id: &UserID,
        range: &Range,
    ) -> Result<Vec<Block>, DbError> {
        self.state.get_user_blocks(user_id, range).await
    }

    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        self.state.insert_friend_request(request).await
    }

    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), DbError> {
        self.state.insert_friendship(user_a, user_b).await
    }

    async fn insert_block(&self, block: &Block) -> Result<(), DbError> {
        self.state.insert_block(block).await
    }

    async fn insert_user(&self, user: &User) -> Result<(), DbError> {
        self.state.insert_user(user).await
    }

    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        self.state.update_friend_request(request).await
    }

    async fn update_user(&self, user_id: &UserID, request: &UpdateUser) -> Result<(), DbError> {
        self.state.update_user(user_id, request).await
    }

    async fn delete_block(&self, request: &Block) -> Result<(), DbError> {
        self.state.delete_block(request).await
    }

    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), DbError> {
        self.state.delete_friendship(friendship).await
    }

    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), DbError> {
        self.state.delete_friend_request(request).await
    }

    async fn delete_user(&self, user: &User) -> Result<(), DbError> {
        self.state.delete_user(user).await
    }
}

#[async_trait]
impl Transaction for InMemoryUserRepositoryTx<'_> {
    async fn commit(self: Box<Self>) -> Result<(), DbError> {
        self.parent.replace_with(&self.state).await;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), DbError> {
        // Discard changes by dropping the snapshot.
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn sample_users() -> Vec<User> {
    vec![
        User {
            id: "user-1".to_string(),
            username: "alice".to_string(),
            created_at: None,
        },
        User {
            id: "user-2".to_string(),
            username: "bob".to_string(),
            created_at: None,
        },
    ]
}
