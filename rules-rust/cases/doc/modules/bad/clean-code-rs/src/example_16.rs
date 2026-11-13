use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

const AUTO_HIDE_DELAY: Duration = Duration::from_secs(3);

pub(crate) struct Banner;

impl Banner {
    pub(crate) fn hide(&self) {}
}

pub(crate) struct TaskScope;

impl TaskScope {
    pub(crate) fn spawn<F: Future<Output = ()> + Send + 'static>(&self, _task: F) {}
}

pub(crate) struct BannerComponent {
    banner: Arc<Banner>,
    task_scope: TaskScope,
}

impl BannerComponent {
    pub(crate) fn schedule_hide(&self, is_visible: bool) {
        let banner = Arc::clone(&self.banner);
        self.task_scope.spawn(async move {
            if is_visible {
                tokio::time::sleep(AUTO_HIDE_DELAY).await;
                banner.hide();
            }
        });
    }
}
