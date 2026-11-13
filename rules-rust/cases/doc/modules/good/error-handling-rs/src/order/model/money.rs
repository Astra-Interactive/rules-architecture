#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Money(i64);

impl Money {
    pub(crate) const ZERO: Money = Money(0);
}
