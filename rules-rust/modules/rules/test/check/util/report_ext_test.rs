use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::Report;
use crate::check::model::RuleId;
use std::path::PathBuf;
use std::process::ExitCode;

fn build_report(violations: Vec<Finding>) -> Report {
    Report {
        violations,
        accepted: Vec::new(),
        target_directory: PathBuf::from("target"),
    }
}

#[test]
fn given_findings_when_exit_code_is_asked_then_returns_one() {
    let report = build_report(vec![Finding {
        rule_id: RuleId("UnitReadme".to_owned()),
        location: Location {
            path: "modules/sample".to_owned(),
            line: None,
        },
        subject: "case-sample".to_owned(),
        message: "no README.md".to_owned(),
    }]);

    assert!(!report.is_clean());
    assert_eq!(ExitCode::from(1), report.to_exit_code());
}

#[test]
fn given_no_findings_when_exit_code_is_asked_then_returns_success() {
    let report = build_report(Vec::new());

    assert!(report.is_clean());
    assert_eq!(ExitCode::SUCCESS, report.to_exit_code());
    assert_eq!("", report.to_violation_lines());
}
