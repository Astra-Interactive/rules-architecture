use std::time::SystemTime;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckoutStage {
    Payment,
}

#[derive(Clone)]
pub(crate) struct Cart {
    pub(crate) expires_at: SystemTime,
}

#[derive(Clone)]
pub(crate) struct CheckoutContent {
    pub(crate) stage: CheckoutStage,
    pub(crate) is_busy: bool,
    pub(crate) cart: Cart,
}

pub(crate) enum CheckoutState {
    Empty,
    Content(CheckoutContent),
}

pub(crate) fn start_checkout(current: CheckoutState, now: SystemTime) -> CheckoutState {
    if let CheckoutState::Content(content) = &current
        && content.stage == CheckoutStage::Payment
        && !content.is_busy
        && content.cart.expires_at > now
    {
        return CheckoutState::Content(CheckoutContent {
            is_busy: true,
            ..content.clone()
        });
    }
    current
}
