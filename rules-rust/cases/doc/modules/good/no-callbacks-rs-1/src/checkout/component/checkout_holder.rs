use crate::checkout::model::CheckoutEvent;
use tokio::sync::mpsc;

pub(crate) struct CheckoutHolder {
    events: mpsc::Sender<CheckoutEvent>,
}

impl CheckoutHolder {
    pub(crate) fn new(events: mpsc::Sender<CheckoutEvent>) -> Self {
        Self { events }
    }

    pub(crate) fn on_paid(&self) {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "a full buffer drops the event, as Kotlin's trySend does; a gone consumer has stopped"
        )]
        let _ = self.events.try_send(CheckoutEvent::Paid);
    }
}
