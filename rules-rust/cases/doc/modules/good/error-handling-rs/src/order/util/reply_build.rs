use crate::order::model::{Order, OrderError, OrderReply};
use crate::order::util::save_order;
use std::error::Error;

pub(crate) fn build_reply(result: Result<Order, OrderError>, order: Order) -> OrderReply {
    let reply = match result {
        Ok(order) => OrderReply::Accepted(order.id),
        Err(error) => OrderReply::Rejected(error.to_string()),
    };

    if let Err(error) = save_order(order) {
        tracing::warn!(error = &error as &dyn Error, "Could not save order");
    }
    reply
}
