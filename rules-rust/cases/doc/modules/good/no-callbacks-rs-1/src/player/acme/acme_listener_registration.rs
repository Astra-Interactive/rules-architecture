use acme_sdk::AcmePlayer;
use std::sync::Arc;

pub(crate) struct AcmeListenerRegistration {
    player: Arc<AcmePlayer>,
    listener_id: u64,
}

impl AcmeListenerRegistration {
    pub(crate) fn new(player: Arc<AcmePlayer>, listener_id: u64) -> Self {
        Self {
            player,
            listener_id,
        }
    }
}

impl Drop for AcmeListenerRegistration {
    fn drop(&mut self) {
        self.player.remove_state_listener(self.listener_id);
    }
}
