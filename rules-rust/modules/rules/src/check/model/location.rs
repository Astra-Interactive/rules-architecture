#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Location {
    pub path: String,
    pub line: Option<u32>,
}
