use std::time::Duration;

use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations/postgres");

pub struct PgOptions<'a> {
    pub url: &'a str,
    pub max_conns: u32,
    pub acquire_timeout: Duration,
}

pub async fn new_pg_pool<'a>(config: &'a PgOptions<'a>) -> anyhow::Result<PgPool> {
    let db = PgPoolOptions::new()
        .max_connections(config.max_conns)
        .acquire_timeout(config.acquire_timeout)
        .connect(config.url)
        .await?;

    MIGRATOR.run(&db).await;

    Ok(db)
}
