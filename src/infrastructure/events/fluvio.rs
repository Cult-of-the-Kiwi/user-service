use fluvio::Fluvio;
use serde::Serialize;

use crate::application::events::publisher::EventPublisher;

impl<T: Serialize> EventPublisher<T> for Fluvio {
    type Event;

    fn publish(&self, event: T) {
        todo!()
    }
    
}