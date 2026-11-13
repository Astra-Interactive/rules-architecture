// timer/memory/in_memory_timer.rs: the stream behind the contract
use crate::timer::api::Timer;
use crate::timer::model::TimerEvent;
use futures::stream::{self, BoxStream, StreamExt};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

pub(crate) struct InMemoryTimer {
    events: broadcast::Sender<TimerEvent>,
}

impl InMemoryTimer {
    pub(crate) fn new(capacity: usize) -> Self {
        let (events, _) = broadcast::channel(capacity);
        Self { events }
    }
}

impl Timer for InMemoryTimer {
    fn events(&self) -> BoxStream<'static, TimerEvent> {
        stream::unfold(self.events.subscribe(), |mut receiver| async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => return Some((event, receiver)),
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => return None,
                }
            }
        })
        .boxed()
    }
}
