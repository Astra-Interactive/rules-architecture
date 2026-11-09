use crate::pricing::model::Money;
use std::ops::Mul;

/// Computes the discount amount, excluding promotional items.
///
/// `rate` is a fraction in the range `0.0..=1.0`, not a percentage. The amount is rounded to cents,
/// half up.
///
/// # Panics
///
/// Panics if `rate` is outside `0.0..=1.0`.
pub fn compute_discount(price: Money, rate: f64) -> Money {
    assert!(
        (0.0..=1.0).contains(&rate),
        "rate {rate} is outside 0.0..=1.0"
    );
    let discount = f64::from(price.cents()).mul(rate).round();
    Money::new(discount as i32)
}
