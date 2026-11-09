// invoice/model/money.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Money(i64);

impl Money {
    pub(crate) fn new(cents: i64) -> Self {
        assert!(cents >= 0, "money is never negative, got {cents}");
        Self(cents)
    }

    pub(crate) fn cents(&self) -> i64 {
        self.0
    }

    pub(crate) const ZERO: Money = Money(0);
}
