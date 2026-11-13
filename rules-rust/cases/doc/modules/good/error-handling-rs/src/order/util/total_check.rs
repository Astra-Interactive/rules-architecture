use crate::order::model::{Money, OrderError};
use crate::order::util::parse_order;

pub(crate) fn is_total_zero(raw: &str) -> bool {
    let total: Result<Money, OrderError> = parse_order(raw).map(|order| order.total);

    let total_or_zero: Money = total.unwrap_or(Money::ZERO);
    total_or_zero == Money::ZERO
}
