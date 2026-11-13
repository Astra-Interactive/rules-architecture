use crate::reminder::service::{InvoiceReminder, TaskScope};
use std::sync::Arc;

pub(crate) struct ReminderModule {
    invoice_reminder: Arc<InvoiceReminder>,
    task_scope: Arc<TaskScope>,
}

impl ReminderModule {
    pub(crate) fn new() -> Self {
        Self {
            invoice_reminder: Arc::new(InvoiceReminder::new()),
            task_scope: Arc::new(TaskScope::new()),
        }
    }

    pub(crate) fn start_reminder(&self) {
        let reminder = Arc::clone(&self.invoice_reminder);
        self.task_scope.spawn(async move { reminder.run().await });
    }
}
