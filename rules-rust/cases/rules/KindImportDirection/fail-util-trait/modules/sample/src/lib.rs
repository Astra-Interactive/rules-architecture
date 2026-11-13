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
    use crate::api::Repository;
    pub(crate) fn compute_page(repository: &dyn crate::api::Repository) {}
}
