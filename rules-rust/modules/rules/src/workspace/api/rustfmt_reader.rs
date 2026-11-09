use crate::workspace::model::RustfmtFacts;

pub(crate) trait RustfmtReader: Send + Sync {
    fn read_rustfmt_facts(&self) -> RustfmtFacts;
}
