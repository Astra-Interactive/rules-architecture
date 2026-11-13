use crate::check::di::CheckModule;
use crate::check::model::{Anchor, Finding, Location, RuleFinding, RuleId};
use crate::source::model::{ExpectationPosition, ModuleId, NodeSpan, TextPosition};
use crate::suppression::model::{ExpectationSlot, RuleActivity, ValidatedExpectations};
use crate::suppression::util::apply_suppressions;
use crate::test::check::usecase::read_case_context;
use std::collections::BTreeSet;
use std::fs;

pub(crate) fn build_workspace() -> tempfile::TempDir {
    let context = read_case_context("ForbiddenPackageName", "pass-names");
    let temporary = tempfile::tempdir().expect("temporary workspace");
    for file in &context.workspace.files {
        let destination = temporary.path().join(file);
        fs::create_dir_all(destination.parent().expect("file parent")).expect("fixture directory");
        fs::copy(context.workspace.root.join(file), destination).expect("copy fixture");
    }
    temporary
}

#[test]
fn given_inline_cfg_siblings_when_inner_approved_then_only_its_own_declaration_is_covered() {
    let temporary = build_workspace();
    let root = temporary.path().join("modules/sample/src/lib.rs");
    for (second, count) in [
        ("", 1),
        (
            "#![cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = \"Windows approved\"))]",
            0,
        ),
    ] {
        let source = format!(
            "pub(crate) mod sample {{ pub(crate) mod util {{ #[cfg(unix)] mod common {{ #![cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = \"Unix approved\"))] }} #[cfg(windows)] mod common {{ {second} }} }} }}"
        );
        fs::write(&root, source).expect("write inline declarations");
        let report = CheckModule::default()
            .check_workspace_use_case
            .invoke(temporary.path())
            .expect("valid workspace");
        let rendered = report.to_violation_lines();
        assert_eq!(
            report
                .violations
                .iter()
                .filter(|finding| finding.rule_id.0 == "ForbiddenPackageName")
                .count(),
            count
        );
        assert!(
            !report
                .violations
                .iter()
                .any(|finding| finding.rule_id.0.contains("Expectation")),
            "{rendered}"
        );
    }
}

#[test]
fn given_cfg_siblings_loading_one_file_when_inner_approved_then_both_instances_are_covered() {
    let temporary = build_workspace();
    let root = temporary.path().join("modules/sample/src/lib.rs");
    let common = temporary
        .path()
        .join("modules/sample/src/sample/util/std.rs");
    fs::create_dir_all(common.parent().expect("module parent")).expect("module directory");
    fs::write(common, "#![cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = \"shared body approved\"))] pub(crate) fn read_values() {}").expect("write common body");
    fs::write(root, "pub(crate) mod sample { pub(crate) mod util { #[cfg(unix)] mod std; #[cfg(windows)] mod std; } }").expect("write cfg declarations");
    let report = CheckModule::default()
        .check_workspace_use_case
        .invoke(temporary.path())
        .expect("valid workspace");
    let rendered = report.to_violation_lines();
    assert!(
        !report
            .violations
            .iter()
            .any(|finding| finding.rule_id.0 == "ForbiddenPackageName"
                || finding.rule_id.0.contains("Expectation")),
        "{rendered}"
    );
    assert_eq!(
        report
            .accepted
            .iter()
            .filter(|finding| finding.rule_id.0 == "ForbiddenPackageName")
            .count(),
        1
    );
    fs::write(temporary.path().join("modules/sample/src/lib.rs"), "pub(crate) mod sample { pub(crate) mod util { #[cfg(unix)] mod std; #[cfg(windows)] mod std {} } }").expect("write mixed cfg declarations");
    let report = CheckModule::default()
        .check_workspace_use_case
        .invoke(temporary.path())
        .expect("valid workspace");
    let rendered = report.to_violation_lines();
    assert_eq!(
        report
            .violations
            .iter()
            .filter(|finding| finding.rule_id.0 == "ForbiddenPackageName")
            .count(),
        1
    );
    assert!(
        !report
            .violations
            .iter()
            .any(|finding| finding.rule_id.0.contains("Expectation")),
        "{rendered}"
    );
}

#[test]
fn given_cfg_siblings_and_macro_closure_when_checked_then_each_approval_has_its_own_scope() {
    let temporary = build_workspace();
    let root = temporary.path().join("modules/sample/src/lib.rs");
    let source = "pub(crate) mod sample { pub(crate) mod util { #[cfg(unix)] #[cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = \"Unix approved\"))] mod std; #[cfg(windows)] SECOND mod std; } }";
    let common = temporary
        .path()
        .join("modules/sample/src/sample/util/std.rs");
    fs::create_dir_all(common.parent().expect("common parent")).expect("module directory");
    fs::write(&common, "pub(crate) fn read_values() { let _closures = vec![#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = \"approved closure\"))] |value: (u8, u8)| value]; }").expect("write shared module");
    for (second, count) in [
        ("", 1),
        (
            "#[cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason = \"Windows approved\"))]",
            0,
        ),
    ] {
        fs::write(&root, source.replace("SECOND", second)).expect("write cfg declarations");
        let report = CheckModule::default()
            .check_workspace_use_case
            .invoke(temporary.path())
            .expect("valid workspace");
        let rendered = report.to_violation_lines();
        assert_eq!(
            report
                .violations
                .iter()
                .filter(|finding| finding.rule_id.0 == "ForbiddenPackageName")
                .count(),
            count
        );
        assert!(
            !report
                .violations
                .iter()
                .any(|finding| finding.rule_id.0.contains("Expectation")),
            "{rendered}"
        );
        assert_eq!(
            report
                .accepted
                .iter()
                .filter(|finding| finding.rule_id.0 == "AnonymousTuple")
                .count(),
            1
        );
    }
}

fn build_span(first: u32, last: u32) -> NodeSpan {
    NodeSpan {
        start: TextPosition {
            line: first,
            column: 0,
        },
        end: TextPosition {
            line: last,
            column: 0,
        },
    }
}

fn build_slot(position: ExpectationPosition, reason: &str) -> ExpectationSlot {
    ExpectationSlot {
        rule_id: "AnonymousTuple".to_owned(),
        file: "src/value.rs".to_owned(),
        line: 1,
        position,
        reason: reason.to_owned(),
    }
}

#[test]
fn given_repeated_module_paths_when_inner_approved_then_shared_module_body_remains_covered() {
    let context = read_case_context("AnonymousTuple", "fail-positions");
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: vec!["common".to_owned()],
    };
    let findings: Vec<_> = [2, 4]
        .into_iter()
        .map(|line| RuleFinding {
            finding: Finding {
                rule_id: RuleId("ForbiddenPackageName".to_owned()),
                location: Location {
                    path: "src/lib.rs".to_owned(),
                    line: Some(line),
                },
                subject: "sample::common".to_owned(),
                message: "forbidden name".to_owned(),
            },
            anchor: Anchor::Module {
                module: module.clone(),
                file: "src/lib.rs".to_owned(),
                body_file: "src/common.rs".to_owned(),
                declaration: build_span(line, line),
            },
        })
        .collect();
    let activity = RuleActivity {
        active_rules: BTreeSet::from(["ForbiddenPackageName".to_owned()]),
        is_platform_words_empty: false,
    };
    let slot = ExpectationSlot {
        rule_id: "ForbiddenPackageName".to_owned(),
        file: "src/common.rs".to_owned(),
        line: 1,
        position: ExpectationPosition::ModuleInner(module),
        reason: "shared module body".to_owned(),
    };
    let outcome = apply_suppressions(
        findings,
        ValidatedExpectations {
            slots: vec![slot],
            findings: Vec::new(),
        },
        &context.project_config,
        &activity,
    );
    assert_eq!(outcome.accepted.len(), 2);
    assert!(outcome.violations.is_empty());
}

#[test]
fn given_nested_and_unrelated_scopes_when_suppressed_then_innermost_node_wins_in_either_order() {
    let context = read_case_context("AnonymousTuple", "fail-positions");
    let finding = RuleFinding {
        finding: Finding {
            rule_id: RuleId("AnonymousTuple".to_owned()),
            location: Location {
                path: "src/value.rs".to_owned(),
                line: Some(5),
            },
            subject: "value".to_owned(),
            message: "tuple type".to_owned(),
        },
        anchor: Anchor::Node {
            file: "src/value.rs".to_owned(),
            span: build_span(5, 6),
        },
    };
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: vec!["value".to_owned()],
    };
    let activity = RuleActivity {
        active_rules: BTreeSet::from(["AnonymousTuple".to_owned()]),
        is_platform_words_empty: false,
    };
    let slots = vec![
        build_slot(ExpectationPosition::Node(build_span(1, 10)), "outer"),
        build_slot(ExpectationPosition::Node(build_span(4, 7)), "inner"),
        build_slot(ExpectationPosition::ModuleInner(module), "module"),
        build_slot(ExpectationPosition::File, "file"),
    ];
    for slots in [slots.clone(), slots.into_iter().rev().collect()] {
        let outcome = apply_suppressions(
            vec![finding.clone()],
            ValidatedExpectations {
                slots,
                findings: Vec::new(),
            },
            &context.project_config,
            &activity,
        );
        assert_eq!(outcome.accepted.len(), 1);
        assert_eq!(outcome.accepted[0].reason, "inner");
    }
}
