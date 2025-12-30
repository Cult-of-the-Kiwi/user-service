use std::{env, net::SocketAddr};

use axum::serve;
use tokio::net::TcpListener;
use user_service::app::AppBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let router = AppBuilder::default().await?;

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {addr}");

    serve(listener, router.into_make_service()).await?;

    Ok(())
}
