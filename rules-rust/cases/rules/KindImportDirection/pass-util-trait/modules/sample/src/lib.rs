mod api {
    pub(crate) trait Repository {}
    pub(crate) struct Page;
    pub(crate) enum Order {
        Newest,
    }
}
mod util {
    use crate::api::Order;
    use crate::api::Page;
    pub(crate) fn compute_page(page: &Page, order: Order) {}
}
mod usecase {
    use crate::api::Repository;
    pub(crate) fn load_page(repository: &dyn Repository) {}
}
