use std::pin::Pin;

pub type EventSubscriberHdlrFn = 
    Box<dyn (FnMut(Event) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + Send + Sync>;

pub enum Event {
    UserRegistered,
    UserUpdated,
    FriendshipRequested,
    FriendshipRequestAnswered,
}

pub trait EventPublisher {
    fn subscribe(&self, event: Event, listener: EventSubscriberHdlrFn);
    fn unsubscribe(&self, event: Event, listener: EventSubscriberHdlrFn);
    async fn notify(&self, event: Event) -> anyhow::Result<()>;
}