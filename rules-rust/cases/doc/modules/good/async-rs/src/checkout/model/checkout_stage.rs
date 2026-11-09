#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckoutStage {
    Payment,
    Finished,
}
