#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ModuleIdentity {
    pub(crate) index: usize,
    pub(crate) item: usize,
}
