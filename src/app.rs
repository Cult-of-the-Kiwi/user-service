use std::{sync::Arc, time::Duration};

use anyhow::{Context, Ok};
use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use devcord_events::{
    events::Event,
    publisher::{EventManager, topic::fluvio::FluvioHandler},
};
use dotenvy::var;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{api, application::repositories::user_repository::UserRepository};

#[derive(Clone)]
pub(crate) struct AppState {
    pub db: Arc<dyn UserRepository>,
    pub event_manager: Arc<dyn EventManager<Event = Event>>,
}

pub struct AppBuilder {
    db: Option<Box<dyn UserRepository>>,
    event_manager: Option<Box<dyn EventManager<Event = Event>>>,
    cors_layer: Option<CorsLayer>,
    trace_layer: Option<TraceLayer<SharedClassifier<ServerErrorsAsFailures>>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            db: None,
            event_manager: None,
            cors_layer: None,
            trace_layer: None,
        }
    }

    pub async fn with_db_postgres(mut self) -> anyhow::Result<Self> {
        let max_conns: u32 = var("DB_MAX_CONNECTIONS")
            .unwrap_or("1".to_owned())
            .parse()
            .expect("DB_MAX_CONNECTIONS must be a number");

        let db_timeout: u64 = var("DB_POOL_TIMEOUT_SECS")
            .unwrap_or("10".to_owned())
            .parse()
            .expect("DB_POOL_TIMEOUT_SECS must be a number");

        let db = PgPoolOptions::new()
            .max_connections(max_conns)
            .acquire_timeout(Duration::from_secs(db_timeout))
            .connect(
                var("DATABASE_URL")
                    .expect("DATABASE_URL env not set")
                    .trim(),
            )
            .await?;

        self.db = Some(Box::new(db));

        Ok(self)
    }

    pub async fn with_event_manager_fluvio(mut self) -> anyhow::Result<Self> {
        self.event_manager = Some(Box::new(FluvioHandler::new()?));

        Ok(self)
    }

    pub fn with_cors_layer_env(mut self) -> anyhow::Result<Self> {
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

        self.cors_layer = Some(cors_layer);
        Ok(self)
    }

    pub fn with_trace_layer(mut self) -> Self {
        self.trace_layer = Some(TraceLayer::new_for_http());

        self
    }

    pub fn build(self) -> anyhow::Result<Router> {
        let state = AppState {
            db: Arc::from(self.db.context("Mising app database")?),
            event_manager: Arc::from(self.event_manager.context("Mising app event manager")?),
        };

        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();

        let trace_layer = TraceLayer::new_for_http();

        let mut router = api::new();
        if let Some(cors) = self.cors_layer {
            router = router.layer(cors);
        }
        let router = router.layer(trace_layer).with_state(state.into());

        Ok(router)
    }

    pub async fn default() -> anyhow::Result<Router> {
        Self::new()
            .with_db_postgres()
            .await?
            .with_event_manager_fluvio()
            .await?
            .with_cors_layer_env()?
            .with_trace_layer()
            .build()
    }
}
