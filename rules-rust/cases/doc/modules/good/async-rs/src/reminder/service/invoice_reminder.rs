use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

pub(crate) struct InvoiceReminder {
    reminded: AtomicU32,
}

impl InvoiceReminder {
    pub(crate) fn new() -> Self {
        Self {
            reminded: AtomicU32::new(0),
        }
    }

    pub(crate) async fn run(&self) {
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.reminded.fetch_add(1, Ordering::Relaxed);
    }
}
