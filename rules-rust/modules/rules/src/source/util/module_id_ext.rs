use crate::source::model::ModuleId;

impl ModuleId {
    pub(crate) fn is_test_root(&self) -> bool {
        matches!(self.path.as_slice(), [word] if word == "test")
    }
}
