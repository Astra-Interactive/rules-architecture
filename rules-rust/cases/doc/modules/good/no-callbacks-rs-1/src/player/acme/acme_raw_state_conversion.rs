use crate::player::model::PlaybackState;
use acme_sdk::AcmeRawState;

impl From<AcmeRawState> for PlaybackState {
    fn from(raw_state: AcmeRawState) -> Self {
        if raw_state.code == 0 {
            return PlaybackState::Idle;
        }
        PlaybackState::Playing
    }
}
