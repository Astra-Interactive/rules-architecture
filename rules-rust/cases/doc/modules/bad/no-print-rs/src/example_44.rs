use std::fmt;

#[derive(Debug)]
pub(crate) struct OrderError;

impl fmt::Display for OrderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("order error")
    }
}

pub(crate) struct Order;

pub(crate) fn load_order(order_id: u64) -> Result<Order, OrderError> {
    if order_id == 0 {
        return Err(OrderError);
    }
    Ok(Order)
}

pub(crate) fn run_order_report(order_id: u64) {
    if let Err(error) = load_order(order_id) {
        eprintln!("Could not load order: {error}");
        dbg!(&error);
    }
}
