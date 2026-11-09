use std::future::Future;
use std::sync::Arc;
use tokio::sync::watch;

pub(crate) struct PurchaseOrderUseCase;

impl PurchaseOrderUseCase {
    pub(crate) async fn invoke(&self, _order_id: u64) {
        tokio::task::yield_now().await;
    }
}

pub(crate) struct TaskScope;

impl TaskScope {
    pub(crate) fn spawn<F: Future<Output = ()> + Send + 'static>(&self, _task: F) {}
}

pub(crate) struct CheckoutState {
    pub(crate) order_id: u64,
}

pub(crate) struct CheckoutComponent {
    state: watch::Sender<CheckoutState>,
    task_scope: TaskScope,
    purchase_order_use_case: Arc<PurchaseOrderUseCase>,
}

impl CheckoutComponent {
    pub(crate) fn on_purchase(&self) {
        let purchase_order_use_case = Arc::clone(&self.purchase_order_use_case);
        let order_id = self.state.borrow().order_id;
        self.task_scope.spawn(async move {
            purchase_order_use_case.invoke(order_id).await;
        });
    }
}
