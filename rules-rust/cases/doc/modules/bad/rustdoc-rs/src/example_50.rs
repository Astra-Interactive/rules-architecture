use crate::money::Money;

/// Function for computing a discount. Takes a price and a rate and returns the discount.
pub fn compute_discount(price: Money, rate: f64) -> Money {
    Money(price.0 - (f64::from(price.0) * rate) as i32)
}
