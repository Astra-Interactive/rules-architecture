use crate::timer::model::TimerEvent;
use futures::stream::BoxStream;

pub(crate) trait TimerPublisher: Send + Sync {
    fn events(&self) -> BoxStream<'static, TimerEvent>;
}
