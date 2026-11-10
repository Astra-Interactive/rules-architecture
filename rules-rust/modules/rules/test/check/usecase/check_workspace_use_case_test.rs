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

fn find_fixture_line(example: &Value, block_line: u32) -> Option<String> {
    example
        .get("fragment")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find_map(|fragment| {
            let block = fragment
                .get("block-lines")
                .and_then(Value::as_str)
                .and_then(read_range)?;
            let fixture = fragment
                .get("fixture-lines")
                .and_then(Value::as_str)
                .and_then(read_range)?;
            let path = fragment.get("path").and_then(Value::as_str)?;
            let line = fixture.start() + (block_line.checked_sub(*block.start())?);
            block
                .contains(&block_line)
                .then(|| format!("{path}:{line}"))
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
            let block_line =
                u32::try_from(expected.get("line").and_then(Value::as_integer)?).ok()?;
            let location = find_fixture_line(example, block_line)?;
            is_rule.then(|| format!("{carrier}\t{location}"))
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
    let example = Value::Table("claim=\"case\"\nhalf=\"bad\"\nexpected=[{line=2,carrier=\"AnonymousTuple\"}]\nfragment=[{path=\"src/value.rs\",block-lines=\"1-2\",fixture-lines=\"4-5\"}]".parse().expect("example TOML"));
    let failures = compare_rule_carriers(&[example], &[BTreeSet::new()]);
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("case bad"));
    assert!(failures[0].contains("src/value.rs:5"));
}

#[test]
fn given_v4_module_worked_example_when_checked_then_matches_its_complete_report() {
    let directory = find_rules_rust_root().join("cases/worked/module-kinds");
    let mismatches = compare_case(&directory);

    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
