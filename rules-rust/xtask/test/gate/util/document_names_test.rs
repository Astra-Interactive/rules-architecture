use crate::gate::model::GateError;
use crate::gate::util::collect_document_names;
use std::fs;

#[test]
fn given_directory_entry_io_failure_when_collected_then_the_failure_is_propagated() {
    let root = tempfile::tempdir().expect("temporary directory");
    fs::write(root.path().join("rule.rs.md"), "# Rule").expect("document");
    let entries = fs::read_dir(root.path())
        .expect("directory")
        .chain(std::iter::once(Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "entry cannot be read",
        ))));
    assert!(matches!(
        collect_document_names(root.path(), entries),
        Err(GateError::Read { .. })
    ));
}
