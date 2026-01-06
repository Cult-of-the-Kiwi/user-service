use std::{env, net::SocketAddr};

use anyhow::bail;
use axum::serve;
use tokio::net::TcpListener;
use user_service::app::AppBuilder;

#[derive(Debug, Clone, Copy)]
enum DbBackend {
    Postgres,
    Memory,
}

#[derive(Debug, Clone, Copy)]
enum EventBackend {
    Fluvio,
    Memory,
}

fn parse_args() -> anyhow::Result<(DbBackend, EventBackend)> {
    let mut db = DbBackend::Postgres;
    let mut events = EventBackend::Fluvio;

    for arg in env::args().skip(1) {
        if arg == "--help" || arg == "-h" {
            println!("Usage: user-service [--db=postgres|memory] [--events=fluvio|memory]");
            std::process::exit(0);
        }
        if let Some(value) = arg.strip_prefix("--db=") {
            db = match value {
                "postgres" => DbBackend::Postgres,
                "memory" => DbBackend::Memory,
                other => {
                    bail!("Invalid --db option: {other}. Use postgres or memory");
                }
            };
            continue;
        }
        if let Some(value) = arg.strip_prefix("--events=") {
            events = match value {
                "fluvio" => EventBackend::Fluvio,
                "memory" => EventBackend::Memory,
                other => {
                    bail!("Invalid --events option: {other}. Use fluvio or memory");
                }
            };
            continue;
        }
        bail!("Unknown argument: {arg}");
    }

    Ok((db, events))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let (db_backend, event_backend) = parse_args()?;

    let builder = AppBuilder::default();
    let builder = match db_backend {
        DbBackend::Postgres => builder.with_db_postgres().await?,
        DbBackend::Memory => builder.with_db_in_memory().await?,
    };

    let builder = match event_backend {
        EventBackend::Fluvio => builder.with_event_manager_fluvio().await?,
        EventBackend::Memory => builder.with_event_manager_in_memory()?,
    };

    let router = builder
        .with_cors_layer_env()?
        .with_trace_layer()
        .build()
        .await?;

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse()?;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {addr}");

    serve(listener, router.into_make_service()).await?;

    Ok(())
}
