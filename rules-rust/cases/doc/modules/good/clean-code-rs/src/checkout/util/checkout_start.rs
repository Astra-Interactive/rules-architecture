use crate::checkout::model::{CheckoutContent, CheckoutStage, CheckoutState};
use std::time::SystemTime;

pub(crate) fn start_checkout(current: CheckoutState, now: SystemTime) -> CheckoutState {
    let CheckoutState::Content(content) = &current else {
        return current;
    };
    let is_payment_stage = content.stage == CheckoutStage::Payment;
    let is_idle = !content.is_busy;
    let is_cart_alive = content.cart.expires_at > now;
    if is_payment_stage && is_idle && is_cart_alive {
        return CheckoutState::Content(CheckoutContent {
            is_busy: true,
            ..content.clone()
        });
    }
    current
}
