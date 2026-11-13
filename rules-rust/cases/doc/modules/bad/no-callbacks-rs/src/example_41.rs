pub(crate) struct Money(pub(crate) i64);

pub(crate) struct CheckoutState {
    pub(crate) total: Money,
    pub(crate) show_paid_message: bool,
}
