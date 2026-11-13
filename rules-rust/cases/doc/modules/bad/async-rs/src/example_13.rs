use std::sync::Arc;

pub(crate) struct InvoiceReminder;

impl InvoiceReminder {
    pub(crate) async fn run(&self) {
        tokio::task::yield_now().await;
    }
}

pub(crate) fn start_reminder(reminder: Arc<InvoiceReminder>) {
    tokio::spawn(async move { reminder.run().await });
}
