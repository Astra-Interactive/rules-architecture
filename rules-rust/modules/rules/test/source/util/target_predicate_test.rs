use crate::source::util::{
    find_single_target_value, list_manifest_target_values, list_target_values,
};
use syn::parse::Parser;

#[test]
fn given_target_cfg_syntax_when_read_then_only_documented_predicates_supply_values() {
    for invalid in [
        "unix",
        "cfg(",
        "other(unix)",
        "cfg(unix, windows)",
        "cfg(feature = \"fake\")",
        "cfg(all(target_os = 3))",
        "cfg(all(target_os = env!()))",
        "cfg(other(unix))",
        "cfg(all(,))",
    ] {
        assert!(list_manifest_target_values(invalid).is_empty(), "{invalid}");
    }
    assert_eq!(
        list_manifest_target_values("cfg(all(unix, any(windows, target_arch = \"arm\")))"),
        vec!["unix", "windows", "arm"]
    );
    let attributes = syn::Attribute::parse_outer
        .parse_str("#[cfg(all(unix, windows))] #[cfg(target_os = \"linux\")] #[deprecated]")
        .expect("valid attributes");
    assert_eq!(
        find_single_target_value(&attributes).as_deref(),
        Some("linux")
    );
    assert_eq!(list_target_values(&attributes[0]), vec!["unix", "windows"]);
    let invalid = syn::Attribute::parse_outer
        .parse_str("#[cfg(unix,)]")
        .expect("valid attribute tokens");
    assert!(find_single_target_value(&invalid).is_none());
}
