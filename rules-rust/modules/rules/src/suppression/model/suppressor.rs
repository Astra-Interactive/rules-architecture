#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Suppressor {
    Attribute(usize),
    Manifest(usize),
    Dependency(usize),
}
