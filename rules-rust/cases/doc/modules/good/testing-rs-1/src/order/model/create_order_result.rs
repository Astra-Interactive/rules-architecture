#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreateOrderResult {
    Created,
    CartIsEmpty,
    PaymentDeclined,
}
