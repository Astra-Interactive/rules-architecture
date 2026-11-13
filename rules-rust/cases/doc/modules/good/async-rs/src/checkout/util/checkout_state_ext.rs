// checkout/util/checkout_state_ext.rs: the pure transition
use crate::checkout::model::{CheckoutStage, CheckoutState};

impl CheckoutState {
    pub(crate) fn start_payment(&self) -> Option<CheckoutState> {
        let is_payment_stage = self.stage == CheckoutStage::Payment;
        if !is_payment_stage || self.is_busy {
            return None;
        }
        Some(CheckoutState {
            is_busy: true,
            ..self.clone()
        })
    }
}
