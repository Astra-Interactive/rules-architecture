use klibs_rules::check::di::CheckModule;
use std::path::Path;

#[test]
fn given_this_workspace_when_checked_then_no_rule_is_violated() {
    let check_module = CheckModule::new();

    let report = check_module
        .check_workspace_use_case
        .invoke(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("the klibs-rules configuration loads");

    assert!(report.is_clean(), "{}", report.to_violation_lines());
}
