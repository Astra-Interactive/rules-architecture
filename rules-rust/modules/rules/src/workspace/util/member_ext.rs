use crate::workspace::model::Member;
use crate::workspace::model::Target;

impl Member {
    pub(crate) fn find_lib_target(&self) -> Option<&Target> {
        self.targets.iter().find(|target| target.is_lib_target())
    }

    pub(crate) fn has_rust_library(&self) -> bool {
        self.targets.iter().any(Target::is_rust_library)
    }

    pub(crate) fn is_proc_macro(&self) -> bool {
        self.find_lib_target().is_some_and(Target::is_proc_macro)
    }
}
