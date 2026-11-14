use crate::check::model::WorkspaceArgumentError;
use crate::check::util::parse_workspace_directory;
use std::ffi::OsString;
use std::path::PathBuf;

fn build_arguments(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

#[test]
fn given_accepted_argument_forms_when_workspace_directory_is_parsed_then_returns_the_directory() {
    assert_eq!(
        Ok(PathBuf::from(".")),
        parse_workspace_directory(&build_arguments(&[]))
    );
    assert_eq!(
        Ok(PathBuf::from(".")),
        parse_workspace_directory(&build_arguments(&["klibs-rules"]))
    );
    assert_eq!(
        Ok(PathBuf::from("work/shop")),
        parse_workspace_directory(&build_arguments(&[
            "klibs-rules",
            "--manifest-path",
            "work/shop/Cargo.toml",
        ]))
    );
    assert_eq!(
        Ok(PathBuf::from("work/shop")),
        parse_workspace_directory(&build_arguments(&[
            "--manifest-path",
            "work/shop/Cargo.toml",
        ]))
    );
    assert_eq!(
        Ok(PathBuf::from(".")),
        parse_workspace_directory(&build_arguments(&["--manifest-path", "Cargo.toml"]))
    );
}

#[test]
fn given_any_other_argument_form_when_workspace_directory_is_parsed_then_returns_usage_error() {
    assert_eq!(
        Err(WorkspaceArgumentError::Usage),
        parse_workspace_directory(&build_arguments(&["--manifest-path"]))
    );
    assert_eq!(
        Err(WorkspaceArgumentError::Usage),
        parse_workspace_directory(&build_arguments(&["--bogus"]))
    );
    assert_eq!(
        Err(WorkspaceArgumentError::Usage),
        parse_workspace_directory(&build_arguments(&["klibs-rules", "klibs-rules"]))
    );
    assert_eq!(
        Err(WorkspaceArgumentError::Usage),
        parse_workspace_directory(&build_arguments(&[
            "--manifest-path",
            "work/shop/Cargo.toml",
            "extra",
        ]))
    );
    assert_eq!(
        Err(WorkspaceArgumentError::Usage),
        parse_workspace_directory(&build_arguments(&["--manifest-path", ""]))
    );
}
