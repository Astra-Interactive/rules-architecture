use crate::check::di::CheckModule;
use crate::check::model::CheckError;
use crate::check::model::Report;
use crate::check::model::RuleContext;
use crate::config::util::parse_project_config;
use crate::source::api::SourceReader;
use crate::source::syn::CrateIndexBuilder;
use crate::source::syn::SynSourceReader;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::model::RustfmtFacts;
use crate::workspace::util::resolve_cargo_path;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use toml::Table;
use toml::Value;

const SAMPLE_WORKSPACE_MANIFEST: &str = "[workspace]\nresolver = \"3\"\nmembers = [\"modules/sample\"]\n\n[workspace.metadata.klibs-rules]\nproduct-kind = \"app\"\n";

pub(crate) fn read_case_context(rule: &str, case: &str) -> RuleContext {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/rules")
        .join(rule)
        .join(case);
    let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(&path)
        .expect("the rule case is a valid Cargo workspace");
    let project_config =
        parse_project_config(&workspace).expect("the rule case has valid configuration");
    let crates = SynSourceReader.read_sources(&workspace);
    let indexes = CrateIndexBuilder::build_all(&crates, &workspace);
    RuleContext {
        workspace,
        project_config,
        crates,
        indexes,
        names: std::sync::OnceLock::new(),
        rustfmt: RustfmtFacts::NotQueried,
    }
}

fn find_rules_rust_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn check_workspace(dir: &Path) -> Result<Report, CheckError> {
    CheckModule::default().check_workspace_use_case.invoke(dir)
}

fn is_blessing() -> bool {
    env::var_os("KLIBS_RULES_BLESS").is_some()
}

fn compare_file_with_policy(path: &Path, actual: &str, blessing: bool) -> Option<String> {
    let expected = match fs::read_to_string(path) {
        Ok(expected) => Some(expected),
        Err(error) if !blessing => {
            return Some(format!(
                "could not read snapshot {}: {error}",
                path.display()
            ));
        }
        Err(_) => None,
    };
    if expected.as_deref() == Some(actual) {
        return None;
    }
    if blessing {
        fs::write(path, actual).expect("the expected file is writable");
        return None;
    }
    Some(format!(
        "{}\n--- expected\n{}--- actual\n{actual}",
        path.display(),
        expected.as_deref().unwrap_or("<missing>")
    ))
}

fn compare_file(path: &Path, actual: &str) -> Option<String> {
    compare_file_with_policy(path, actual, is_blessing())
}

fn compare_case(dir: &Path) -> Vec<String> {
    let report = match check_workspace(dir) {
        Ok(report) => report,
        Err(error) => {
            let actual = format!("{error}\n");
            return compare_file(&dir.join("expected-error.txt"), &actual)
                .into_iter()
                .collect();
        }
    };
    let mut mismatches: Vec<String> =
        compare_file(&dir.join("expected.tsv"), &report.to_violation_lines())
            .into_iter()
            .collect();
    let accepted = report.to_policy_lines();
    let accepted_path = dir.join("accepted.tsv");
    if !accepted.is_empty() || accepted_path.exists() {
        mismatches.extend(compare_file(&accepted_path, &accepted));
    }
    mismatches
}

fn list_case_directories(root: &Path) -> Vec<PathBuf> {
    let mut cases: Vec<PathBuf> = fs::read_dir(root)
        .expect("the cases directory exists")
        .map(|entry| entry.expect("the cases directory is readable").path())
        .filter(|path| path.join("Cargo.toml").is_file())
        .collect();
    cases.sort();
    cases
}

fn compare_cases(root: &Path) -> Vec<String> {
    list_case_directories(root)
        .iter()
        .flat_map(|dir| compare_case(dir))
        .collect()
}

#[test]
fn given_rule_cases_when_checked_then_each_report_equals_its_expected_lines() {
    let rules = find_rules_rust_root().join("cases/rules");
    let mut rule_directories: Vec<PathBuf> = fs::read_dir(&rules)
        .expect("cases/rules exists")
        .map(|entry| entry.expect("cases/rules is readable").path())
        .collect();
    rule_directories.sort();

    let mismatches: Vec<String> = rule_directories
        .iter()
        .flat_map(|rule| compare_cases(rule))
        .collect();

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn given_documented_binary_entry_point_when_every_rule_runs_then_nothing_reports() {
    let case = find_rules_rust_root().join("cases/rules/CrateTargets/pass-binary-entry-point");
    let report =
        check_workspace(&case).expect("the documented binary entry point is a valid workspace");

    assert_eq!("", report.to_violation_lines());
}

#[test]
fn given_suppression_cases_when_checked_then_findings_and_accepted_exceptions_equal_the_expected_lines()
 {
    let mismatches = compare_cases(&find_rules_rust_root().join("cases/suppression"));

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn given_configuration_cases_when_checked_then_each_stops_with_its_configuration_error() {
    let mismatches = compare_cases(&find_rules_rust_root().join("cases/configuration"));

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn given_consumer_cases_when_checked_then_the_failing_one_reports_and_the_fixed_one_is_clean() {
    let mismatches = compare_cases(&find_rules_rust_root().join("cases/consumer"));

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

fn read_range(range: &str) -> Option<std::ops::RangeInclusive<u32>> {
    let (first, last) = range.split_once('-')?;
    Some(first.parse().ok()?..=last.parse().ok()?)
}

fn locate_finding(examples: &[Value], path: &str, line: u32) -> Option<usize> {
    examples.iter().position(|example| {
        example
            .get("fragment")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|fragment| {
                let is_file = fragment.get("path").and_then(Value::as_str) == Some(path);
                let range = fragment
                    .get("fixture-lines")
                    .and_then(Value::as_str)
                    .and_then(read_range);
                is_file && range.is_some_and(|range| range.contains(&line))
            })
    })
}

fn list_expected_rule_findings(example: &Value) -> BTreeSet<String> {
    example
        .get("expected")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|expected| {
            let carrier = expected.get("carrier").and_then(Value::as_str)?;
            let is_rule = carrier.chars().next().is_some_and(char::is_uppercase);
            let path = expected.get("path").and_then(Value::as_str)?;
            let line = u32::try_from(expected.get("line").and_then(Value::as_integer)?).ok()?;
            is_rule.then(|| format!("{carrier}\t{path}:{line}"))
        })
        .collect()
}

fn compare_rule_carriers(examples: &[Value], actual: &[BTreeSet<String>]) -> Vec<String> {
    examples
        .iter()
        .zip(actual)
        .filter(|(example, found)| list_expected_rule_findings(example) != **found)
        .map(|(example, found)| {
            format!(
                "{} {}: expected {:?}, found {:?}",
                example
                    .get("claim")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                example
                    .get("half")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                list_expected_rule_findings(example),
                found
            )
        })
        .collect()
}

#[test]
fn given_doc_gate_when_checked_then_each_half_reports_exactly_its_rule_carriers() {
    let doc = find_rules_rust_root().join("cases/doc");
    let examples_text =
        fs::read_to_string(doc.join("examples.toml")).expect("examples.toml exists");
    let examples_table: Table = examples_text.parse().expect("examples.toml parses");
    let examples = examples_table
        .get("example")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let report = check_workspace(&doc).expect("the gate workspace loads");

    let mut actual: Vec<BTreeSet<String>> = vec![BTreeSet::new(); examples.len()];
    let mut scaffold = String::new();
    for finding in &report.violations {
        let line = finding.location.line.unwrap_or(0);
        let entry = format!("{}\t{}:{line}", finding.rule_id.0, finding.location.path);
        match locate_finding(&examples, &finding.location.path, line) {
            Some(index) => {
                actual[index].insert(entry);
            }
            None => scaffold.push_str(&format!(
                "{}\t{}:{line}\t{}\t{}\n",
                finding.rule_id.0, finding.location.path, finding.subject, finding.message
            )),
        }
    }
    let mut mismatches = compare_rule_carriers(&examples, &actual);
    mismatches.extend(compare_file(&doc.join("expected-scaffold.tsv"), &scaffold));

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn given_snapshot_drift_when_compared_then_reports_it_and_only_explicit_blessing_rewrites() {
    let root = tempfile::tempdir().expect("temporary directory");
    let path = root.path().join("expected.tsv");
    assert!(compare_file_with_policy(&path, "new", false).is_some());
    assert!(!path.exists());
    assert!(compare_file_with_policy(&path, "new", true).is_none());
    assert_eq!(fs::read_to_string(path).expect("blessed snapshot"), "new");
}

#[test]
fn given_missing_or_unreadable_snapshot_when_compared_then_even_empty_reports_fail() {
    let root = tempfile::tempdir().expect("temporary directory");
    assert!(
        compare_file_with_policy(&root.path().join("missing.tsv"), "", false)
            .expect("missing snapshot is rejected")
            .contains("could not read snapshot")
    );
    assert!(
        compare_file_with_policy(root.path(), "", false)
            .expect("directory is not a snapshot")
            .contains("could not read snapshot")
    );
    let path = root.path().join("expected.tsv");
    fs::write(&path, "old").expect("old snapshot");
    assert!(
        compare_file_with_policy(&path, "new", false)
            .expect("drift is rejected")
            .contains("--- actual")
    );
}

#[test]
fn given_missing_rule_carrier_when_compared_then_failure_identifies_its_example_and_location() {
    let example = Value::Table("claim=\"case\"\nhalf=\"bad\"\nexpected=[{path=\"src/value.rs\",line=5,carrier=\"AnonymousTuple\"}]\nfragment=[{path=\"src/value.rs\",fixture-lines=\"4-5\"}]".parse().expect("example TOML"));
    let failures = compare_rule_carriers(&[example], &[BTreeSet::new()]);
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("case bad"));
    assert!(failures[0].contains("src/value.rs:5"));
}

fn list_case_roots() -> Vec<PathBuf> {
    let root = find_rules_rust_root().join("cases");
    let mut directories = Vec::new();
    for group in ["rules", "suppression", "consumer"] {
        let group_root = root.join(group);
        if group != "rules" {
            directories.extend(list_case_directories(&group_root));
            continue;
        }
        let rules: Vec<PathBuf> = fs::read_dir(&group_root)
            .expect("the case group exists")
            .map(|entry| entry.expect("the case group is readable").path())
            .collect();
        for rule in rules {
            directories.extend(list_case_directories(&rule));
        }
    }
    directories.sort();
    directories
}

#[test]
fn given_every_case_workspace_when_checked_then_no_message_is_a_placeholder_naming_its_own_rule() {
    let mut offenders = Vec::new();
    for directory in list_case_roots() {
        let Ok(report) = check_workspace(&directory) else {
            continue;
        };
        for line in report.to_violation_lines().lines() {
            let columns: Vec<&str> = line.split('\t').collect();
            let [rule, _, _, message] = columns[..] else {
                continue;
            };
            if message.contains("Follow the") || message.contains(rule) {
                offenders.push(format!("{}: {line}", directory.display()));
            }
        }
    }
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}

struct WorkspaceFile<'file> {
    path: &'file str,
    text: &'file str,
}

fn build_rust_files_workspace(files: &[WorkspaceFile<'_>]) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("temporary workspace");
    for file in files {
        let path = directory.path().join(file.path);
        fs::create_dir_all(path.parent().expect("file parent")).expect("file directory");
        fs::write(path, file.text).expect("workspace file");
    }
    directory
}

#[test]
fn given_source_finding_taken_by_an_exception_when_its_test_twin_exists_then_the_twin_is_not_reported()
 {
    let directory = build_rust_files_workspace(&[
        WorkspaceFile {
            path: "Cargo.toml",
            text: "[workspace]\nresolver = \"3\"\nmembers = [\"modules/sample\"]\n\n[workspace.metadata.klibs-rules]\nproduct-kind = \"app\"\n",
        },
        WorkspaceFile {
            path: "modules/sample/Cargo.toml",
            text: "[package]\nname = \"sample\"\nedition = \"2024\"\n",
        },
        WorkspaceFile {
            path: "modules/sample/src/lib.rs",
            text: "#[cfg_attr(klibs_rules, expect(PluralPackageName, reason = \"suspected false positive, reported as #42\"))]\nmod options {}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {\n    mod options {}\n}\n",
        },
    ]);
    let report = check_workspace(directory.path()).expect("the workspace is checked");
    let violations: Vec<String> = report
        .to_violation_lines()
        .lines()
        .filter(|line| {
            [
                "PluralPackageName",
                "UnfulfilledExpectation",
                "MalformedExpectation",
            ]
            .iter()
            .any(|rule| line.starts_with(&format!("{rule}\t")))
        })
        .map(str::to_owned)
        .collect();
    assert!(violations.is_empty(), "{violations:#?}");
    let accepted: Vec<String> = report
        .to_policy_lines()
        .lines()
        .filter(|line| line.starts_with("PluralPackageName\t"))
        .map(str::to_owned)
        .collect();
    assert_eq!(1, accepted.len(), "{accepted:#?}");
}

#[test]
fn given_attribute_exception_naming_a_retired_rule_without_successor_when_checked_then_the_finding_says_it_has_no_successor()
 {
    let directory = build_rust_files_workspace(&[
        WorkspaceFile {
            path: "Cargo.toml",
            text: SAMPLE_WORKSPACE_MANIFEST,
        },
        WorkspaceFile {
            path: "modules/sample/Cargo.toml",
            text: "[package]\nname = \"sample\"\nedition = \"2024\"\n",
        },
        WorkspaceFile {
            path: "modules/sample/src/lib.rs",
            text: "#[cfg_attr(klibs_rules, expect(ShellAutomation, reason = \"suspected false positive, reported as #42\"))]\nmod options {}\n",
        },
    ]);
    let report = check_workspace(directory.path()).expect("the workspace is checked");
    let retired: Vec<String> = report
        .to_violation_lines()
        .lines()
        .filter(|line| line.starts_with("RetiredRule\t"))
        .map(str::to_owned)
        .collect();
    assert_eq!(
        vec![
            "RetiredRule\tmodules/sample/src/lib.rs:1\tShellAutomation\t`ShellAutomation` is retired in klibs-rules: it has no successor"
                .to_owned()
        ],
        retired
    );
}

#[test]
fn given_manifest_exception_naming_a_retired_rule_without_successor_when_checked_then_the_configuration_error_says_it_has_no_successor()
 {
    let directory = build_rust_files_workspace(&[
        WorkspaceFile {
            path: "Cargo.toml",
            text: SAMPLE_WORKSPACE_MANIFEST,
        },
        WorkspaceFile {
            path: "modules/sample/Cargo.toml",
            text: "[package]\nname = \"sample\"\nedition = \"2024\"\n\n[package.metadata.klibs-rules.checks.disabled.XtaskAlias]\n\"sample\" = \"a reason\"\n",
        },
        WorkspaceFile {
            path: "modules/sample/src/lib.rs",
            text: "",
        },
    ]);
    let Err(error) = check_workspace(directory.path()) else {
        panic!("a retired rule id in the manifest stops the check");
    };
    assert_eq!(
        "modules/sample/Cargo.toml:5: `XtaskAlias` is retired: it has no successor",
        error.to_string()
    );
}

#[test]
fn given_v4_module_worked_example_when_checked_then_matches_its_complete_report() {
    let directory = find_rules_rust_root().join("cases/worked/module-kinds");
    let mismatches = compare_case(&directory);

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
