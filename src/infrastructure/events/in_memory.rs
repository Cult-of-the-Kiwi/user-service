use std::{collections::HashMap, future::Future, pin::Pin};

use async_trait::async_trait;
use devcord_events::{
    events::{Event, EventType},
    publisher::{EventManager, TypedEvent},
};
use tokio::sync::Mutex;

pub(crate) type Handler =
    Box<dyn FnMut(Event) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync>;

#[derive(Default)]
pub(crate) struct InMemoryEventManager {
    subscribers: Mutex<HashMap<EventType, Vec<Handler>>>,
}

impl InMemoryEventManager {
    pub(crate) fn new() -> Self {
        Self {
            subscribers: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl EventManager for InMemoryEventManager {
    type Event = Event;

    async fn subscribe(&self, event: Self::Event, listener: Handler) -> anyhow::Result<()> {
        let mut map = self.subscribers.lock().await;
        map.entry(event.event_type()).or_default().push(listener);
        Ok(())
    }

    async fn unsubscribe(&self, event: Self::Event) -> anyhow::Result<()> {
        let mut map = self.subscribers.lock().await;
        map.remove(&event.event_type());
        Ok(())
    }

    async fn notify(&self, event: Self::Event) -> anyhow::Result<()> {
        let mut map = self.subscribers.lock().await;
        if let Some(listeners) = map.get_mut(&event.event_type()) {
            for listener in listeners.iter_mut() {
                listener(event.clone()).await;
            }
        }
        Ok(())
    }
}
