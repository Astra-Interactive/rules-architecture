use crate::check::api::ReportWriter;
use crate::check::file::FileReportWriter;
use crate::check::model::AcceptedException;
use crate::check::model::CheckError;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::Report;
use crate::check::model::RuleId;
use std::fs;
use std::path::PathBuf;

fn build_report(target_directory: PathBuf) -> Report {
    Report {
        violations: vec![Finding {
            rule_id: RuleId("UnitReadme".to_owned()),
            location: Location {
                path: "modules/billing/sqlx".to_owned(),
                line: None,
            },
            subject: "shop-billing-sqlx".to_owned(),
            message: "no README.md".to_owned(),
        }],
        accepted: vec![AcceptedException {
            rule_id: RuleId("AnonymousTuple".to_owned()),
            location: Location {
                path: "src/lib.rs".to_owned(),
                line: Some(3),
            },
            subject: "shop".to_owned(),
            reason: "reported to klibs-rules as #1".to_owned(),
        }],
        target_directory,
    }
}

#[test]
fn given_report_when_written_then_violations_and_policy_files_hold_its_lines() {
    let target = tempfile::tempdir().expect("a temporary directory");
    let report = build_report(target.path().to_path_buf());

    FileReportWriter
        .write(&report)
        .expect("the report is written");

    let violations = fs::read_to_string(target.path().join("klibs-rules/violations.txt"))
        .expect("violations.txt");
    let policy =
        fs::read_to_string(target.path().join("klibs-rules/policy.txt")).expect("policy.txt");
    assert_eq!(
        "UnitReadme\tmodules/billing/sqlx\tshop-billing-sqlx\tno README.md\n",
        violations
    );
    assert_eq!(
        "AnonymousTuple\tsrc/lib.rs:3\tshop\treported to klibs-rules as #1\n",
        policy
    );
}

#[test]
fn given_target_directory_that_is_a_file_when_written_then_returns_io_error() {
    let target = tempfile::tempdir().expect("a temporary directory");
    let blocked = target.path().join("blocked");
    fs::write(&blocked, "").expect("a plain file");

    let result = FileReportWriter.write(&build_report(blocked));

    assert!(matches!(result, Err(CheckError::Io { .. })));
}

#[test]
fn given_report_directory_whose_file_is_a_directory_when_written_then_returns_io_error() {
    let target = tempfile::tempdir().expect("a temporary directory");
    fs::create_dir_all(target.path().join("klibs-rules/violations.txt"))
        .expect("a directory in the file's place");

    let result = FileReportWriter.write(&build_report(target.path().to_path_buf()));

    assert!(matches!(result, Err(CheckError::Io { .. })));
}
