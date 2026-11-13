use std::env;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn resolve_cargo_path() -> PathBuf {
    env::var_os("CARGO").map_or(PathBuf::from("cargo"), PathBuf::from)
}

pub(crate) fn resolve_rustfmt_path(cargo: &Path) -> PathBuf {
    let has_directory = cargo
        .parent()
        .is_some_and(|parent| !parent.as_os_str().is_empty());
    if !has_directory {
        return PathBuf::from("rustfmt");
    }
    let file_name = format!("rustfmt{}", env::consts::EXE_SUFFIX);
    cargo.with_file_name(file_name)
}
