use async_trait::async_trait;
use devcord_sqlx_utils::error::Error;

use crate::domain::{
    models::{
        block::Block,
        friend_request::{FriendRequest, FriendRequestDirection, FriendRequestRange},
        friendship::Friendship,
        range::Range,
        update_user::UpdateUser,
        user::User,
    },
    types::UserID,
};

#[async_trait]
pub(crate) trait UserRepository: Send + Sync {
    //Getters
    async fn get_user(&self, user_id: &UserID) -> Result<User, Error>;
    async fn get_user_friend(&self, user_id: &UserID, friend_id: &UserID) -> Result<User, Error>;
    async fn get_friend_request(&self, request: &FriendRequest) -> Result<FriendRequest, Error>;
    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Result<Vec<FriendRequest>, Error>;
    async fn get_user_friends(&self, user_id: &UserID, range: &Range) -> Result<Vec<User>, Error>;
    async fn get_user_block(&self, user_id: &UserID, blocked_id: &UserID) -> Result<User, Error>;
    async fn get_user_blocks(&self, user_id: &UserID, range: &Range) -> Result<Vec<Block>, Error>;
    //Inserts
    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), Error>;
    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), Error>;
    async fn insert_block(&self, block: &Block) -> Result<(), Error>;
    async fn insert_user(&self, user: &User) -> Result<(), Error>;
    //Updates
    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), Error>;
    async fn update_user(&self, user_id: &UserID, request: &UpdateUser) -> Result<(), Error>;
    //DELETE
    async fn delete_block(&self, request: &Block) -> Result<(), Error>;
    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), Error>;
    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), Error>;
    async fn delete_user(&self, user: &User) -> Result<(), Error>;
}
