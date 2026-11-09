use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckoutStage {
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckoutState {
    pub(crate) order_id: u64,
    pub(crate) stage: CheckoutStage,
    pub(crate) is_busy: bool,
}

pub(crate) fn publish(
    state: &watch::Sender<CheckoutState>,
    order_id: u64,
) -> Result<(), watch::error::SendError<CheckoutState>> {
    state.send(CheckoutState { order_id, stage: CheckoutStage::Finished, is_busy: false }) // never
}
