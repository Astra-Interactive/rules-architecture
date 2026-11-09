use crate::checkout::model::{Cart, CheckoutStage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckoutContent {
    pub(crate) stage: CheckoutStage,
    pub(crate) is_busy: bool,
    pub(crate) cart: Cart,
}
