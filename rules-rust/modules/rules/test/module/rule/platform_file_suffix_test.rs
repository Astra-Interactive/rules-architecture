use crate::module::rule::PlatformFileSuffix;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&PlatformFileSuffix::new(), "PlatformFileSuffix", "bad");
    assert_code_fixture(&PlatformFileSuffix::new(), "PlatformFileSuffix", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &PlatformFileSuffix::new(),
        "PlatformFileSuffix",
        "mod api {\n    #[cfg(target_os = \"linux\")]\n    mod clipboard;\n    #[cfg(any(target_os = \"linux\", target_os = \"macos\"))]\n    mod clipboard_other;\n    #[cfg(unix)]\n    fn run() {}\n}",
        &[2, 4, 6],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &PlatformFileSuffix::new(),
        "PlatformFileSuffix",
        "mod api {\n    #[cfg(target_os = \"linux\")]\n    pub(crate) use clipboard_linux::Clipboard;\n    #[cfg(target_os = \"macos\")]\n    pub(crate) use clipboard_macos::Clipboard;\n    #[cfg(target_os = \"linux\")]\n    mod clipboard_linux;\n    #[cfg(target_os = \"macos\")]\n    mod clipboard_macos;\n}",
        &[],
    );
}

#[test]
fn given_wrong_reexport_predicate_or_name_when_checked_then_reports_platform_shape() {
    assert_code_findings(
        &PlatformFileSuffix::new(),
        "PlatformFileSuffix",
        "mod api {\n    pub(crate) use clipboard_linux::Clipboard;\n    #[cfg(target_os = \"linux\")] mod clipboard_linux;\n}",
        &[2],
    );
    assert_code_findings(
        &PlatformFileSuffix::new(),
        "PlatformFileSuffix",
        "mod api {\n    #[cfg(unix)] pub(crate) use clipboard_unix::Clipboard;\n    #[cfg(windows)] pub(crate) use clipboard_windows::OtherClipboard;\n    #[cfg(unix)] mod clipboard_unix;\n    #[cfg(windows)] mod clipboard_windows;\n}",
        &[4, 5],
    );
}
