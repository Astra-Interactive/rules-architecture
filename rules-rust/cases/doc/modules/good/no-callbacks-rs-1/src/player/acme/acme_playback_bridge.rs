// player/acme/acme_playback_bridge.rs, the technology module of the made-up Acme SDK
use crate::player::acme::AcmeListenerRegistration;
use crate::player::api::PlaybackStatePublisher;
use crate::player::model::PlaybackState;
use acme_sdk::AcmePlayer;
use futures::stream::{self, BoxStream, StreamExt};
use parking_lot::Mutex;
use std::sync::{Arc, Weak};
use tokio::sync::watch;

pub(crate) struct AcmePlaybackBridge {
    player: Arc<AcmePlayer>,
    state: Arc<watch::Sender<PlaybackState>>,
    registration: Mutex<Weak<AcmeListenerRegistration>>,
}

impl AcmePlaybackBridge {
    pub(crate) fn new(player: Arc<AcmePlayer>) -> Self {
        let (state, _) = watch::channel(PlaybackState::Idle);
        Self {
            player,
            state: Arc::new(state),
            registration: Mutex::new(Weak::new()),
        }
    }

    fn acquire_registration(&self) -> Arc<AcmeListenerRegistration> {
        let mut registration = self.registration.lock();
        if let Some(active) = registration.upgrade() {
            return active;
        }
        let state = Arc::clone(&self.state);
        let listener_id = self.player.add_state_listener(Box::new(move |raw_state| {
            state.send_replace(PlaybackState::from(raw_state));
        }));
        let active = Arc::new(AcmeListenerRegistration::new(
            Arc::clone(&self.player),
            listener_id,
        ));
        *registration = Arc::downgrade(&active);
        active
    }
}

impl PlaybackStatePublisher for AcmePlaybackBridge {
    fn state(&self) -> BoxStream<'static, PlaybackState> {
        let registration = self.acquire_registration();
        let mut receiver = self.state.subscribe();
        receiver.mark_changed();
        stream::unfold(
            (receiver, registration),
            |(mut receiver, registration)| async move {
                if receiver.changed().await.is_err() {
                    return None;
                }
                let state = receiver.borrow_and_update().clone();
                Some((state, (receiver, registration)))
            },
        )
        .boxed()
    }
}
