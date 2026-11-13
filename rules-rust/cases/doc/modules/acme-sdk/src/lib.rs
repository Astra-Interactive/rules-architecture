use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AcmeError {
    pub message: String,
}

pub struct AcmeStatus {
    pub granted: bool,
}

pub struct AcmeRawState {
    pub code: u8,
}

pub type ConsentCallback = Box<dyn FnOnce(Result<AcmeStatus, AcmeError>) + Send>;

pub type StateListener = Box<dyn Fn(AcmeRawState) + Send + Sync>;

#[derive(Default)]
pub struct Acme {
    next_id: AtomicU64,
    callbacks: Mutex<HashMap<u64, ConsentCallback>>,
}

impl Acme {
    pub fn add_consent_callback(&self, callback: ConsentCallback) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.callbacks.lock().insert(id, callback);
        id
    }

    pub fn remove_consent_callback(&self, id: u64) {
        self.callbacks.lock().remove(&id);
    }

    pub fn request_consent(&self) {
        let callbacks = &self.callbacks;
        let pending: Vec<_> = callbacks.lock().drain().map(|entry| entry.1).collect();
        for callback in pending {
            callback(Ok(AcmeStatus { granted: true }));
        }
    }
}

#[derive(Default)]
pub struct AcmePlayer {
    next_id: AtomicU64,
    listeners: Mutex<HashMap<u64, StateListener>>,
}

impl AcmePlayer {
    pub fn add_state_listener(&self, listener: StateListener) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.listeners.lock().insert(id, listener);
        id
    }

    pub fn remove_state_listener(&self, id: u64) {
        self.listeners.lock().remove(&id);
    }

    pub fn play(&self) {
        for listener in self.listeners.lock().values() {
            listener(AcmeRawState { code: 1 });
        }
    }
}
