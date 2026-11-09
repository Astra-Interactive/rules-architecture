use crate::banner::component::Banner;
use crate::banner::model::AUTO_HIDE_DELAY;
use crate::task::service::TaskScope;
use std::sync::Arc;

pub(crate) struct BannerComponent {
    banner: Arc<Banner>,
    task_scope: Arc<TaskScope>,
}

impl BannerComponent {
    pub(crate) fn new(banner: Arc<Banner>, task_scope: Arc<TaskScope>) -> Self {
        Self { banner, task_scope }
    }

    pub(crate) fn schedule_hide(&self, is_visible: bool) {
        let banner = Arc::clone(&self.banner);
        self.task_scope.spawn(async move {
            if !is_visible {
                return;
            }
            tokio::time::sleep(AUTO_HIDE_DELAY).await;
            banner.hide();
        });
    }
}
