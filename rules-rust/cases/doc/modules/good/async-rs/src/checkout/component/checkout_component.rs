use crate::checkout::model::{CheckoutStage, CheckoutState};
use crate::checkout::usecase::PurchaseOrderUseCase;
use crate::reminder::service::TaskScope;
use std::sync::Arc;
use tokio::sync::watch;

pub(crate) struct CheckoutComponent {
    state: watch::Sender<CheckoutState>,
    task_scope: Arc<TaskScope>,
    purchase_order_use_case: Arc<PurchaseOrderUseCase>,
}

impl CheckoutComponent {
    pub(crate) fn new(
        state: watch::Sender<CheckoutState>,
        task_scope: Arc<TaskScope>,
        purchase_order_use_case: Arc<PurchaseOrderUseCase>,
    ) -> Self {
        Self {
            state,
            task_scope,
            purchase_order_use_case,
        }
    }

    pub(crate) fn is_busy(&self) -> bool {
        let receiver = self.state.subscribe();
        let content = receiver.borrow().clone();
        content.is_busy
    }

    pub(crate) fn finish(&self) {
        let state = &self.state;
        let order_id = state.borrow().order_id;
        state.send_modify(|current| {
            *current = CheckoutState {
                is_busy: false,
                ..current.clone()
            }
        }); // depends on it
        state.send_replace(CheckoutState {
            order_id,
            stage: CheckoutStage::Finished,
            is_busy: false,
        }); // does not
    }

    // the state holder
    fn claim_payment(&self) -> Option<CheckoutState> {
        let mut started = None;
        self.state.send_if_modified(|current| {
            let Some(next) = current.start_payment() else {
                return false;
            };
            *current = next.clone();
            started = Some(next);
            true
        });
        started
    }

    pub(crate) fn on_purchase(&self) {
        let Some(started) = self.claim_payment() else {
            return;
        };
        let purchase_order_use_case = Arc::clone(&self.purchase_order_use_case);
        self.task_scope.spawn(async move {
            purchase_order_use_case.invoke(started.order_id).await;
        });
    }
}
