use crate::workspace::api::RustfmtReader;
use crate::workspace::cargo::CargoRustfmtReader;
use crate::workspace::model::RustfmtFacts;
use crate::workspace::util::resolve_cargo_path;
use crate::workspace::util::resolve_rustfmt_path;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[test]
fn given_toolchain_rustfmt_when_read_then_nightly_only_keys_exclude_stable_ones() {
    let reader = CargoRustfmtReader::new(resolve_rustfmt_path(&resolve_cargo_path()));

    let facts = reader.read_rustfmt_facts();

    assert!(
        matches!(facts, RustfmtFacts::Queried { nightly_only_keys } if nightly_only_keys.contains("group_imports") && !nightly_only_keys.contains("max_width"))
    );
}

#[cfg(unix)]
#[test]
fn given_formatter_supporting_defaults_but_not_help_when_read_then_reports_missing_capability() {
    let root = tempfile::tempdir().expect("temporary directory");
    let command = root.path().join("rustfmt");
    fs::write(&command, "#!/bin/sh\nif [ \"$1\" = \"--print-config=default\" ]; then printf 'max_width = 100\\n'; else exit 1; fi\n").expect("command fixture");
    fs::set_permissions(&command, fs::Permissions::from_mode(0o755)).expect("executable");
    assert!(matches!(
        CargoRustfmtReader::new(command).read_rustfmt_facts(),
        RustfmtFacts::Missing { .. }
    ));
}

#[test]
fn given_missing_rustfmt_when_read_then_names_the_binary() {
    let reader = CargoRustfmtReader::new(PathBuf::from("/nonexistent/rustfmt"));

    let facts = reader.read_rustfmt_facts();

    assert_eq!(
        RustfmtFacts::Missing {
            binary: "/nonexistent/rustfmt".to_owned()
        },
        facts
    );
}

#[test]
fn given_rustfmt_that_rejects_the_config_options_when_read_then_counts_as_missing() {
    let reader = CargoRustfmtReader::new(resolve_cargo_path());

    let facts = reader.read_rustfmt_facts();

    assert!(matches!(facts, RustfmtFacts::Missing { .. }));
}
