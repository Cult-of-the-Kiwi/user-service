use crate::{
    infrastructure::repositories::in_memory::InMemoryUserRepository,
    tests::user_repository::{
        delete_block_ok, delete_friend_request_ok, delete_friendship_ok, delete_user_ok,
        get_friendships, insert_block_ok, insert_friend_request_duplicate,
        insert_friend_request_ok, insert_friendship_ok, insert_user_duplicate_id,
        insert_user_duplicate_name, insert_user_ok, transaction_commit_persists,
        transaction_rollback_discards, update_user_name, update_user_name_without_changing_others,
    },
};

fn repo() -> InMemoryUserRepository {
    InMemoryUserRepository::new(vec![])
}

// -------------------- USER TESTS --------------------
#[tokio::test]
async fn test_in_memory_insert_user_succeeds() {
    insert_user_ok(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_insert_user_fails_when_name_taken() {
    insert_user_duplicate_name(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_insert_user_fails_when_id_taken() {
    insert_user_duplicate_id(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_update_user_name() {
    update_user_name(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_update_user_name_without_changing_others() {
    update_user_name_without_changing_others(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_delete_user_ok() {
    delete_user_ok(&repo()).await;
}

// -------------------- FRIEND REQUEST TESTS --------------------
#[tokio::test]
async fn test_in_memory_insert_friend_request_succeeds() {
    insert_friend_request_ok(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_insert_friend_request_fails_when_duplicate() {
    insert_friend_request_duplicate(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_delete_friend_request_ok() {
    delete_friend_request_ok(&repo()).await;
}

// -------------------- FRIENDSHIP TESTS --------------------
#[tokio::test]
async fn test_in_memory_insert_friendship_ok() {
    insert_friendship_ok(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_delete_friendship_ok() {
    delete_friendship_ok(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_get_friendships() {
    get_friendships(&repo()).await;
}

// -------------------- BLOCK TESTS --------------------
#[tokio::test]
async fn test_in_memory_insert_block_ok() {
    insert_block_ok(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_delete_block_ok() {
    delete_block_ok(&repo()).await;
}

// -------------------- TRANSACTION TESTS --------------------
#[tokio::test]
async fn test_in_memory_transaction_commit_persists() {
    transaction_commit_persists(&repo()).await;
}

#[tokio::test]
async fn test_in_memory_transaction_rollback_discards() {
    transaction_rollback_discards(&repo()).await;
}
