use crate::order::model::OrderOutcome;
use tokio::sync::oneshot;

pub(crate) fn deliver_outcome(sender: oneshot::Sender<OrderOutcome>, outcome: OrderOutcome) {
    #[expect(
        clippy::let_underscore_must_use,
        reason = "the receiver is gone only when the caller was cancelled"
    )]
    let _ = sender.send(outcome);
}
