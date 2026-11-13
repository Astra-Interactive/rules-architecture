use crate::order::model::OrderId;
use crate::order::util::load_order;

pub(crate) fn run_order_report(order_id: OrderId) {
    if let Err(error) = load_order(order_id) {
        tracing::warn!(
            error = &error as &dyn std::error::Error,
            "Could not load order"
        );
    }
}
