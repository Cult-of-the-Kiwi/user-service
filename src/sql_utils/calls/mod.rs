pub(crate) mod postgres;

use devcord_sqlx_utils::error::Error;

use crate::api_utils::{
    structs::{
        Block, FriendRequest, FriendRequestDirection, FriendRequestRange, Friendship, Range,
        UpdateUser, User,
    },
    types::UserID,
};

pub trait UserRepository {
    //Getters
    async fn get_user(&self, user_id: &UserID) -> Option<User>;
    async fn get_user_friend(&self, user_id: &UserID, friend_id: &UserID) -> Option<User>;
    async fn get_friend_request(&self, request: &FriendRequest) -> Option<FriendRequest>;
    async fn get_friend_requests(
        &self,
        user_id: &UserID,
        range: &FriendRequestRange,
        direction: &FriendRequestDirection,
    ) -> Option<Vec<FriendRequest>>;
    async fn get_user_friends(&self, user_id: &UserID, range: &Range) -> Option<Vec<User>>;
    async fn get_user_block(&self, user_id: &UserID, blocked_id: &UserID) -> Option<User>;
    async fn get_user_blocks(&self, user_id: &UserID, range: &Range) -> Option<Vec<Block>>;
    //Inserts
    async fn insert_friend_request(&self, request: &FriendRequest) -> Result<(), Error>;
    async fn insert_friendship(&self, user_a: &UserID, user_b: &UserID) -> Result<(), Error>;
    async fn insert_block(&self, request: &Block) -> Result<(), Error>;
    async fn insert_user(&self, user: &User) -> Result<(), Error>;
    //Updates
    async fn update_friend_request(&self, request: &FriendRequest) -> Result<(), Error>;
    async fn update_user(&self, request: &UpdateUser) -> Result<(), Error>;
    //DELETE
    async fn delete_block(&self, request: &Block) -> Result<(), Error>;
    async fn delete_friendship(&self, friendship: &Friendship) -> Result<(), Error>;
    async fn delete_friend_request(&self, request: &FriendRequest) -> Result<(), Error>;
    async fn delete_user(&self, user: &User) -> Result<(), Error>;
}
