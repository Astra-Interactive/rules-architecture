use crate::checkout::model::OrderId;
use tokio::sync::mpsc;

pub(crate) struct PurchaseOrderUseCase {
    purchases: mpsc::UnboundedSender<OrderId>,
}

impl PurchaseOrderUseCase {
    pub(crate) fn new(purchases: mpsc::UnboundedSender<OrderId>) -> Self {
        Self { purchases }
    }

    pub(crate) async fn invoke(&self, order_id: OrderId) {
        tokio::task::yield_now().await;
        #[expect(
            clippy::let_underscore_must_use,
            reason = "the gate's purchase log has no reader once the test ends"
        )]
        let _ = self.purchases.send(order_id);
    }
}
