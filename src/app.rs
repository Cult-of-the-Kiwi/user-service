use std::{env::var, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
    serve,
};
use fluvio::{Fluvio, TopicProducer, spu::SpuSocketPool};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    application::repositories::user_repository::UserRepository,
    handlers::{
        block::{block, get_blocks, unblock},
        friendship::{
            friend_requests::{
                accept_request, get_requests_received, get_requests_sent, request_friend,
            },
            friends::{get_friends, remove_friend},
        },
        user::{get_user, update},
    },
    infrastructure::context::{
        db::postgres::{PgOptions, new_pg_pool},
        events::fluvio::new_fluvio,
    },
};

#[derive(Clone)]
pub(crate) struct AppState<T: UserRepository> {
    pub db: T,
    pub request_sent_producer: TopicProducer<SpuSocketPool>,
    pub request_answered_producer: TopicProducer<SpuSocketPool>,
}

pub async fn app<T: UserRepository>(fluvio: Fluvio, db: T) -> anyhow::Result<Router> {
    let origins: Vec<HeaderValue> = var("CORS_ORIGIN")
        .expect("CORS_ORIGIN env not set")
        .split(",")
        .map(|e| e.trim().parse::<HeaderValue>())
        .collect::<Result<_, _>>()?;

    let cors_layer = CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
        ]);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let trace_layer = TraceLayer::new_for_http();

    let request_producer_topic = var("USER_RESQUEST_TOPIC")
        .unwrap_or("friendships-request".to_owned())
        .trim()
        .to_string();

    let answered_producer_topic = var("USER_ANSWER_TOPIC")
        .unwrap_or("friendships-answer".to_owned())
        .trim()
        .to_string();

    let request_producer = fluvio.topic_producer(request_producer_topic).await?;

    let answered_producer = fluvio.topic_producer(answered_producer_topic).await?;

    let state = Arc::new(AppState {
        db,
        request_sent_producer: request_producer,
        request_answered_producer: answered_producer,
    });

    let friendships_router = Router::new()
        .route("/request", post(request_friend))
        .route("/accept", post(accept_request))
        .route("/reject", post(accept_request))
        .route("/sent", get(get_requests_sent))
        .route("/received", get(get_requests_received))
        .route("/friends", get(get_friends))
        .route("/unfriend", get(remove_friend));

    let block_router = Router::new()
        .route("/block", post(block))
        .route("/unblock", post(unblock))
        .route("/", get(get_blocks));

    let app = Router::new()
        .nest("/friendship", friendships_router)
        .nest("/blocks", block_router)
        .route("/update", post(update))
        .route("/", get(get_user))
        .route(
            "/health",
            get(|| async { "Long life to the allmighty turbofish" }),
        )
        .layer(cors_layer)
        .layer(trace_layer)
        .with_state(state);

    Ok(app)
}

pub async fn run() -> anyhow::Result<()> {
    let max_conns: u32 = var("DB_MAX_CONNECTIONS")
        .unwrap_or("1".to_owned())
        .parse()
        .expect("DB_MAX_CONNECTIONS must be a number");

    let db_timeout: u64 = var("DB_POOL_TIMEOUT_SECS")
        .unwrap_or("10".to_owned())
        .parse()
        .expect("DB_POOL_TIMEOUT_SECS must be a number");

    let db_url = var("DATABASE_URL")
        .expect("DATABASE_URL env not set")
        .trim()
        .to_string();

    let options = PgOptions {
        url: &db_url,
        max_conns: max_conns,
        acquire_timeout: Duration::from_secs(db_timeout),
    };
    let db = new_pg_pool(&options).await?;

    let fluvio = new_fluvio(var("FLUVIO_ADDR").expect("FLUVIO_ADDR env not set").trim()).await?;

    let app = app(fluvio, db).await?;

    let addr: SocketAddr = var("SOCKET_ADDR")
        .expect("SOCKET_ADDR env not set")
        .parse()?;
    let listener = TcpListener::bind(addr).await?;

    println!("Server runnnig at: {addr}");


    serve(listener, app.into_make_service()).await?;
    Ok(())
}
