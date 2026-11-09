use std::time::Duration;
use tokio::sync::watch;
use tokio::time::timeout;

pub(crate) async fn read_state(receiver: &mut watch::Receiver<u32>) -> bool {
    let content = timeout(Duration::from_secs(1), receiver.wait_for(|_| true)).await;
    content.is_ok()
}
