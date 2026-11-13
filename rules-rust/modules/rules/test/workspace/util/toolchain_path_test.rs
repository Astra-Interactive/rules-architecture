use crate::workspace::util::resolve_rustfmt_path;
use std::env;
use std::path::Path;
use std::path::PathBuf;

#[test]
fn given_cargo_from_path_when_rustfmt_is_resolved_then_uses_rustfmt_from_path() {
    let rustfmt = resolve_rustfmt_path(Path::new("cargo"));

    assert_eq!(PathBuf::from("rustfmt"), rustfmt);
}

#[test]
fn given_cargo_of_a_toolchain_when_rustfmt_is_resolved_then_uses_the_rustfmt_beside_it() {
    let rustfmt = resolve_rustfmt_path(Path::new("/toolchain/bin/cargo"));

    assert_eq!(
        PathBuf::from(format!("/toolchain/bin/rustfmt{}", env::consts::EXE_SUFFIX)),
        rustfmt
    );
}
