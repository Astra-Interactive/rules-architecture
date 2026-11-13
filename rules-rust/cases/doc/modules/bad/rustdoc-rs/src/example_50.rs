use crate::money::Money;

impl Money {
    /// Method for computing a discount. Takes a rate and returns the discount.
    pub fn compute_discount(&self, rate: f64) -> Money {
        Money(self.0 - (f64::from(self.0) * rate) as i32)
    }
}
