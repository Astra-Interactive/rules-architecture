use std::ops::Mul;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Money(f64);

impl Money {
    pub(crate) const ZERO: Money = Money(0.0);
}

impl Mul<f64> for Money {
    type Output = Money;

    fn mul(self, factor: f64) -> Money {
        Money(self.0 * factor)
    }
}
