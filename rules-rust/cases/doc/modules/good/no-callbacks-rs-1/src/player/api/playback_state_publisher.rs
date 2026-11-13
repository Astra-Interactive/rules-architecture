use crate::player::model::PlaybackState;
use futures::stream::BoxStream;

pub(crate) trait PlaybackStatePublisher: Send + Sync {
    fn state(&self) -> BoxStream<'static, PlaybackState>;
}
