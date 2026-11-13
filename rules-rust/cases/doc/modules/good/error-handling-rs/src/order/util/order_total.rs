use crate::order::model::{Money, OrderError};
use crate::order::util::parse_order;

fn compute_order_total(raw: &str) -> Result<Money, OrderError> {
    let order = parse_order(raw)?;
    let open_order = order.ensure_open()?;
    Ok(open_order.total)
}
