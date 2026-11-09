use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct Banner {
    is_shown: AtomicBool,
}

impl Banner {
    pub(crate) fn new() -> Self {
        Self {
            is_shown: AtomicBool::new(true),
        }
    }

    pub(crate) fn hide(&self) {
        self.is_shown.store(false, Ordering::Relaxed);
    }
}
