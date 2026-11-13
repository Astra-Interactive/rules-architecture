use crate::gate::model::ExampleEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExamplesFile {
    pub(crate) commit: String,
    pub(crate) entries: Vec<ExampleEntry>,
}
