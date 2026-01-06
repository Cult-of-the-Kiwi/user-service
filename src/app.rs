use std::{sync::Arc, time::Duration};

use anyhow::{Context, Ok};
use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use devcord_events::{
    events::{
        Event,
        auth::{AuthEvent, UserCreated},
    },
    publisher::{EventManager, topic::fluvio::FluvioHandler},
};
use dotenvy::var;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::warn;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{api, application::repositories::user_repository::UserRepository};
use crate::{
    domain::models::user::User,
    infrastructure::{
        context::db::postgres::{PgOptions, new_pg_pool},
        repositories::postgres::PostgresUserRepository,
    },
};

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

        let db = new_pg_pool(&PgOptions {
            url: var("DATABASE_URL")
                .expect("DATABASE_URL env not set")
                .trim(),
            max_conns,
            acquire_timeout: Duration::from_secs(db_timeout),
        })
        .await?;

        self.db = Some(Box::new(PostgresUserRepository::new(db)));

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

    pub async fn build(self) -> anyhow::Result<Router> {
        let state = Arc::new(AppState {
            db: Arc::from(self.db.context("Mising app database")?),
            event_manager: Arc::from(self.event_manager.context("Mising app event manager")?),
        });

        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();

        let trace_layer = TraceLayer::new_for_http();

        let mut router = api::new();
        if let Some(cors) = self.cors_layer {
            router = router.layer(cors);
        }
        let router = router.layer(trace_layer).with_state(state.clone());

        register_event_listeners(state.db.clone(), state.event_manager.clone()).await?;

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
            .await
    }
}

pub(crate) async fn register_event_listeners(
    db: Arc<dyn UserRepository>,
    event_manager: Arc<dyn EventManager<Event = Event>>,
) -> anyhow::Result<()> {
    let user_created = Event::AuthEvent(AuthEvent::UserSignedUpEvent(UserCreated {
        id: String::new(),
        username: String::new(),
    }));

    event_manager
        .subscribe(
            user_created,
            Box::new(move |event| {
                let db = db.clone();
                Box::pin(async move {
                    if let Event::AuthEvent(AuthEvent::UserSignedUpEvent(payload)) = event {
                        let user = User {
                            id: payload.id,
                            username: payload.username,
                            created_at: None,
                        };
                        if let Err(e) = db.insert_user(&user).await {
                            warn!("Failed to insert user from auth event: {e:?}");
                        }
                    }
                })
            }),
        )
        .await?;

    Ok(())
}
