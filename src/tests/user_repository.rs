use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{
    GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::time::sleep;

use crate::sql_utils::{calls::UserRepository, init::init};

#[tokio::test]
pub async fn test_postgres() {
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
        .expect("Failt to initialize postgres db");

    let port = pg.get_host_port_ipv4(5432).await.unwrap();
    sleep(Duration::from_secs(10)).await;
    let db_url = format!("postgres://postgres:password@127.0.0.1:{}/testdb", port);

    let db = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to postgres db");

    init(&db).await.unwrap();

    test_insert_user_ok(&db).await;
}

pub async fn test_insert_user_ok<T: UserRepository>(db: &T) {
    use crate::api_utils::structs::User;

    let user = User {
        username: "Name A".to_owned(),
        id: "Test ID".to_owned(),
        created_at: None,
    };
    let res = db.insert_user(&user).await;

    assert!(res.is_ok());
    let user_get = db.get_user(&user.id).await;
    assert_eq!(Some(user), user_get)
}
