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
fn given_ownership_advice_when_written_then_three_files_preserve_findings_and_exit_semantics() {
    let target = tempfile::tempdir().expect("temporary directory");
    let checked = crate::check::model::CheckedWorkspace {
        report: build_report(target.path().to_owned()),
        api_ownership: crate::check::model::ApiOwnershipReport {
            text: "CANDIDATE\tValue\n".to_owned(),
        },
    };
    let violations = checked.report.to_violation_lines();
    let policy = checked.report.to_policy_lines();
    FileReportWriter
        .write_with_ownership(&checked)
        .expect("three report files");
    let directory = target.path().join("klibs-rules");
    assert_eq!(
        violations,
        fs::read_to_string(directory.join("violations.txt")).unwrap()
    );
    assert_eq!(
        policy,
        fs::read_to_string(directory.join("policy.txt")).unwrap()
    );
    assert_eq!(
        "CANDIDATE\tValue\n",
        fs::read_to_string(directory.join("api-ownership.txt")).unwrap()
    );
    assert_eq!(
        std::process::ExitCode::from(1),
        checked.report.to_exit_code()
    );
    assert_eq!(3, fs::read_dir(directory).unwrap().count());
}

#[test]
fn given_legacy_writer_implementing_only_write_when_ownership_is_requested_then_existing_caller_still_works()
 {
    struct LegacyWriter;
    impl ReportWriter for LegacyWriter {
        fn write(&self, report: &Report) -> Result<(), CheckError> {
            assert_eq!(1, report.violations.len());
            Ok(())
        }
    }
    let target = tempfile::tempdir().unwrap();
    let checked = crate::check::model::CheckedWorkspace {
        report: build_report(target.path().to_owned()),
        api_ownership: crate::check::model::ApiOwnershipReport {
            text: String::new(),
        },
    };
    LegacyWriter.write_with_ownership(&checked).unwrap();
    assert!(!target.path().join("klibs-rules").exists());
}

#[test]
fn given_unwritable_advisory_path_when_written_then_the_third_file_error_is_returned() {
    let target = tempfile::tempdir().unwrap();
    let blocked = target.path().join("klibs-rules/api-ownership.txt");
    fs::create_dir_all(&blocked).unwrap();
    let checked = crate::check::model::CheckedWorkspace {
        report: build_report(target.path().to_owned()),
        api_ownership: crate::check::model::ApiOwnershipReport {
            text: String::new(),
        },
    };
    assert!(
        matches!(FileReportWriter.write_with_ownership(&checked),Err(CheckError::Io{path,..}) if path.ends_with("api-ownership.txt"))
    );
}

#[test]
fn given_report_directory_whose_file_is_a_directory_when_written_then_returns_io_error() {
    let target = tempfile::tempdir().expect("a temporary directory");
    fs::create_dir_all(target.path().join("klibs-rules/violations.txt"))
        .expect("a directory in the file's place");

    let result = FileReportWriter.write(&build_report(target.path().to_path_buf()));

    assert!(matches!(result, Err(CheckError::Io { .. })));
}
