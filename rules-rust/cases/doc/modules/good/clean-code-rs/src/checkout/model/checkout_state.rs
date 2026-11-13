use crate::checkout::model::CheckoutContent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckoutState {
    Empty,
    Content(CheckoutContent),
}
