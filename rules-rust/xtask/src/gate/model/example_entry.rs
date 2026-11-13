#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExampleEntry {
    pub(crate) document: String,
    pub(crate) anchor: String,
    pub(crate) first_line: String,
    pub(crate) occurrence: usize,
    pub(crate) wrap: String,
    pub(crate) sha256: String,
    pub(crate) fragments: Vec<ExampleFragment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExampleFragment {
    pub(crate) first_block_line: usize,
    pub(crate) last_block_line: usize,
    pub(crate) suffix: String,
    pub(crate) translations: Vec<Translation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Translation {
    pub(crate) from: String,
    pub(crate) to: String,
}
