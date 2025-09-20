use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::time::sleep;

use crate::{
    infrastructure::context::db::postgres::{PgOptions, new_pg_pool},
    tests::user_repository::{
        delete_block_ok, delete_friend_request_ok, delete_friendship_ok, delete_user_ok,
        get_friendships, insert_block_ok, insert_friend_request_duplicate,
        insert_friend_request_ok, insert_friendship_ok, insert_user_duplicate_id,
        insert_user_duplicate_name, insert_user_ok, update_user_name,
        update_user_name_without_changing_others,
    },
};

async fn setup_postgres() -> (ContainerAsync<GenericImage>, PgPool) {
    let pg = GenericImage::new("postgres", "16-alpine")
        .with_exposed_port(ContainerPort::Tcp(5432))
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "testdb")
        .start()
        .await
        .expect("Failed to start postgres container");

    let port = pg.get_host_port_ipv4(5432).await.unwrap();
    sleep(Duration::from_secs(2)).await;

    let options = PgOptions {
        url: &format!("postgres://postgres:password@127.0.0.1:{}/testdb", port),
        max_conns: 1,
        acquire_timeout: Duration::from_secs(10),
    };
    let db = new_pg_pool(&options).await.unwrap();

    (pg, db)
}

// -------------------- USER TESTS --------------------
#[tokio::test]
async fn test_postgres_insert_user_succeeds() {
    let (_pg, db) = setup_postgres().await;
    insert_user_ok(&db).await;
}

#[tokio::test]
async fn test_postgres_insert_user_fails_when_name_taken() {
    let (_pg, db) = setup_postgres().await;
    insert_user_duplicate_name(&db).await;
}

#[tokio::test]
async fn test_postgres_insert_user_fails_when_id_taken() {
    let (_pg, db) = setup_postgres().await;
    insert_user_duplicate_id(&db).await;
}

#[tokio::test]
async fn test_postgres_update_user_name() {
    let (_pg, db) = setup_postgres().await;
    update_user_name(&db).await;
}

#[tokio::test]
async fn test_postgres_update_user_name_without_changing_others() {
    let (_pg, db) = setup_postgres().await;
    update_user_name_without_changing_others(&db).await;
}

#[tokio::test]
async fn test_postgres_delete_user_ok() {
    let (_pg, db) = setup_postgres().await;
    delete_user_ok(&db).await;
}

// -------------------- FRIEND REQUEST TESTS --------------------
#[tokio::test]
async fn test_postgres_insert_friend_request_succeeds() {
    let (_pg, db) = setup_postgres().await;
    insert_friend_request_ok(&db).await;
}

#[tokio::test]
async fn test_postgres_insert_friend_request_fails_when_duplicate() {
    let (_pg, db) = setup_postgres().await;
    insert_friend_request_duplicate(&db).await;
}

#[tokio::test]
async fn test_postgres_delete_friend_request_ok() {
    let (_pg, db) = setup_postgres().await;
    delete_friend_request_ok(&db).await;
}

// -------------------- FRIENDSHIP TESTS --------------------
#[tokio::test]
async fn test_postgres_insert_friendship_ok() {
    let (_pg, db) = setup_postgres().await;
    insert_friendship_ok(&db).await;
}

#[tokio::test]
async fn test_postgres_delete_friendship_ok() {
    let (_pg, db) = setup_postgres().await;
    delete_friendship_ok(&db).await;
}

#[tokio::test]
async fn test_postgres_get_friendships() {
    let (_pg, db) = setup_postgres().await;
    get_friendships(&db).await;
}

// -------------------- BLOCK TESTS --------------------
#[tokio::test]
async fn test_postgres_insert_block_ok() {
    let (_pg, db) = setup_postgres().await;
    insert_block_ok(&db).await;
}

#[tokio::test]
async fn test_postgres_delete_block_ok() {
    let (_pg, db) = setup_postgres().await;
    delete_block_ok(&db).await;
}
