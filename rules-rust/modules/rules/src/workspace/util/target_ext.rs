use crate::workspace::model::Target;

impl Target {
    pub(crate) fn is_lib_target(&self) -> bool {
        self.kinds
            .iter()
            .any(|kind| Self::LIB_KINDS.contains(&kind.as_str()))
    }

    pub(crate) fn is_rust_library(&self) -> bool {
        self.kinds
            .iter()
            .any(|kind| Self::RUST_LIBRARY_KINDS.contains(&kind.as_str()))
    }

    pub(crate) fn is_proc_macro(&self) -> bool {
        self.kinds.iter().any(|kind| kind == "proc-macro")
    }

    pub(crate) fn is_bin(&self) -> bool {
        self.kinds.iter().any(|kind| kind == "bin")
    }

    const LIB_KINDS: [&str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];
    const RUST_LIBRARY_KINDS: [&str; 4] = ["lib", "rlib", "dylib", "proc-macro"];
}
