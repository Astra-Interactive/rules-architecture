use crate::config::util::render_required_configuration;
use cargo_metadata::Message;
use cargo_metadata::diagnostic::Diagnostic;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use toml::Table;
use toml::Value;

const PROBES: &str = r#"probes = [
  { file = "probes.rs", item = "probe_unreachable_pub", lint = "unreachable_pub" },
  { file = "probes.rs", item = "probe_unexpected_cfgs", lint = "unexpected_cfgs" },
  { file = "probes.rs", item = "probe_silent_klibs_rules_cfg" },
  { file = "probes.rs", item = "declare_probe_ids!" },
  { file = "probe_mod_rs/mod.rs", item = "", lint = "clippy::mod_module_files" },
  { file = "probe_named.rs", item = "", lint = "clippy::self_named_module_files" },
  { file = "probes.rs", item = "probe_let_underscore_must_use", lint = "clippy::let_underscore_must_use" },
  { file = "probes.rs", item = "probe_unused_result_ok", lint = "clippy::unused_result_ok" },
  { file = "probes.rs", item = "probe_unwrap_used", lint = "clippy::unwrap_used" },
  { file = "probes.rs", item = "probe_expect_used", lint = "clippy::expect_used" },
  { file = "probes.rs", item = "probe_print_stdout", lint = "clippy::print_stdout" },
  { file = "probes.rs", item = "probe_print_stderr", lint = "clippy::print_stderr" },
  { file = "probes.rs", item = "probe_dbg_macro", lint = "clippy::dbg_macro" },
  { file = "probes.rs", item = "probe_allow_attributes", lint = "clippy::allow_attributes" },
  { file = "probes.rs", item = "probe_allow_attributes_without_reason", lint = "clippy::allow_attributes_without_reason" },
  { file = "probes.rs", item = "probe_too_many_lines", lint = "clippy::too_many_lines" },
  { file = "probes.rs", item = "probe_undocumented_unsafe_blocks", lint = "clippy::undocumented_unsafe_blocks" },
  { file = "probes.rs", item = "probe_redundant_closure_for_method_calls", lint = "clippy::redundant_closure_for_method_calls" },
  { file = "probes.rs", item = "probe_std_mutex", lint = "clippy::disallowed_types" },
  { file = "probes.rs", item = "probe_std_rw_lock", lint = "clippy::disallowed_types" },
  { file = "probes.rs", item = "probe_poison_error", lint = "clippy::disallowed_types" },
  { file = "probes.rs", item = "probe_thread_spawn", lint = "clippy::disallowed_methods" },
  { file = "probes.rs", item = "probe_thread_builder_spawn", lint = "clippy::disallowed_methods" },
  { file = "probes.rs", item = "probe_io_stdout", lint = "clippy::disallowed_methods" },
  { file = "probes.rs", item = "probe_io_stderr", lint = "clippy::disallowed_methods" },
  { file = "probes.rs", item = "probe_system_time_now", lint = "clippy::disallowed_methods" },
  { file = "probes.rs", item = "probe_system_time_elapsed", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_chrono_utc_now", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_chrono_local_now", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_jiff_timestamp_now", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_jiff_zoned_now", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_time_offset_date_time_now_utc", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_time_offset_date_time_now_local", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_time_utc_date_time_now", lint = "clippy::disallowed_methods" },
  { file = "time/src/probes.rs", item = "probe_deprecated_chrono_utc_today", lint = "deprecated" },
  { file = "time/src/probes.rs", item = "probe_deprecated_chrono_local_today", lint = "deprecated" },
  { file = "probes.rs", item = "probe_watch_sender_send", lint = "clippy::disallowed_methods" },
  { file = "probes.rs", item = "probe_silent_allow_unwrap_in_tests" },
  { file = "probes.rs", item = "probe_silent_allow_expect_in_tests" },
  { file = "probes.rs", item = "probe_excessive_nesting", lint = "clippy::excessive_nesting" },
  { file = "probes.rs", item = "probe_await_holding_watch_ref", lint = "clippy::await_holding_invalid_type" },
  { file = "probes.rs", item = "probe_check_private_items", lint = "clippy::missing_safety_doc" },
]"#;

fn find_doc_gate() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cases/doc")
}

fn find_cargo() -> PathBuf {
    env::var_os("CARGO").map_or(PathBuf::from("cargo"), PathBuf::from)
}

fn find_gate_target_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/klibs-rules-gate")
}

fn read_range(range: &str) -> Option<std::ops::RangeInclusive<u32>> {
    let (first, last) = range.split_once('-')?;
    Some(first.parse().ok()?..=last.parse().ok()?)
}

fn read_examples() -> Vec<Value> {
    let text =
        fs::read_to_string(find_doc_gate().join("examples.toml")).expect("examples.toml exists");
    let table: Table = text.parse().expect("examples.toml parses");
    table
        .get("example")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn find_fragment<'examples>(
    examples: &'examples [Value],
    path: &str,
    line: u32,
) -> Option<&'examples Value> {
    examples
        .iter()
        .flat_map(|example| {
            example
                .get("fragment")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|fragment| {
            let is_file = fragment.get("path").and_then(Value::as_str) == Some(path);
            let range = fragment
                .get("fixture-lines")
                .and_then(Value::as_str)
                .and_then(read_range);
            is_file && range.is_some_and(|range| range.contains(&line))
        })
}

fn assert_toolchain_version(expected: Option<&str>) {
    if let Some(expected) = expected {
        for program in [PathBuf::from("rustc"), find_cargo()] {
            let output = Command::new(&program)
                .arg("--version")
                .current_dir(find_doc_gate())
                .output()
                .expect("the toolchain runs");
            let version = String::from_utf8_lossy(&output.stdout).into_owned();
            assert_eq!(
                (
                    &program,
                    &version,
                    version.contains(&format!(" {expected} "))
                ),
                (&program, &version, true)
            );
        }
    }
}

fn assert_toolchain() {
    assert_toolchain_version(env::var("KLIBS_RULES_EXPECT_TOOLCHAIN").ok().as_deref());
}

#[test]
fn given_no_requested_toolchain_when_validated_then_the_optional_version_check_accepts_it() {
    assert_toolchain_version(None);
}

fn normalise_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            ".." => {
                segments.pop();
            }
            "." | "" => {}
            other => segments.push(other),
        }
    }
    segments.join("/")
}

fn run_clippy() -> Vec<Diagnostic> {
    let output = Command::new(find_cargo())
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--keep-going",
            "--message-format=json",
        ])
        .env("CARGO_TARGET_DIR", find_gate_target_directory())
        .current_dir(find_doc_gate())
        .output()
        .expect("cargo clippy runs");
    Message::parse_stream(BufReader::new(output.stdout.as_slice()))
        .map(|message| message.expect("Cargo emits valid JSON messages"))
        .filter_map(|message| match message {
            Message::CompilerMessage(compiler) => Some(compiler.message),
            _ => None,
        })
        .collect()
}

fn build_probe_path(probe: &Value) -> String {
    let file = probe
        .get("file")
        .and_then(Value::as_str)
        .expect("a probe names its source");
    if file.starts_with("time/") {
        format!("modules/probe/{file}")
    } else {
        format!("modules/probe/std/src/{file}")
    }
}

fn find_probe_range(probe: &Value) -> Option<std::ops::RangeInclusive<u32>> {
    let item = probe.get("item")?.as_str()?;
    let text = fs::read_to_string(find_doc_gate().join(build_probe_path(probe))).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    let total = u32::try_from(lines.len()).ok()?;
    if item.is_empty() {
        return Some(1..=total);
    }
    let is_start = |line: &str| {
        line.contains("fn probe_") || line.trim_start().starts_with("declare_probe_ids!")
    };
    let start = lines.iter().position(|line| {
        line.contains(&format!("fn {item}("))
            || (item.ends_with('!') && line.trim_start().starts_with(item))
    })?;
    let mut first = start;
    while first > 0
        && lines[first - 1]
            .trim_start()
            .starts_with(['#', ')', 'k', 'e'])
    {
        first -= 1;
    }
    let end = lines
        .iter()
        .skip(start + 1)
        .position(|line| is_start(line))
        .map_or(lines.len(), |offset| start + offset);
    let mut last = end;
    while last > start + 1
        && lines[last - 1]
            .trim_start()
            .starts_with(['#', ')', 'k', 'e'])
    {
        last -= 1;
    }
    Some(u32::try_from(first + 1).ok()?..=u32::try_from(last).ok()?)
}

fn read_probes() -> Vec<Value> {
    let table: Table = PROBES.parse().expect("the probe table parses");
    let probes = table
        .get("probes")
        .and_then(Value::as_array)
        .expect("the probe table has records");
    assert_eq!(probes.len(), 42);
    probes.clone()
}

fn check_probes(diagnostics: &[Diagnostic], probes: &[Value]) -> Vec<String> {
    probes
        .iter()
        .filter_map(|probe| {
            let Some(range) = find_probe_range(probe) else {
                return Some(format!("probe {probe:?} not found"));
            };
            let path = build_probe_path(probe);
            let inside: Vec<&Diagnostic> = diagnostics
                .iter()
                .filter(|diagnostic| {
                    let Some(code) = &diagnostic.code else {
                        return false;
                    };
                    if code.code == "dead_code" {
                        return false;
                    }
                    diagnostic
                        .spans
                        .iter()
                        .find(|span| span.is_primary)
                        .is_some_and(|span| {
                            let line = u32::try_from(span.line_start).ok();
                            normalise_path(&span.file_name) == path
                                && line.is_some_and(|line| range.contains(&line))
                        })
                })
                .collect();
            let passes = match probe.get("lint").and_then(Value::as_str) {
                Some(lint) => inside.iter().any(|diagnostic| {
                    diagnostic
                        .code
                        .as_ref()
                        .is_some_and(|code| code.code == lint)
                }),
                None => inside.is_empty(),
            };
            (!passes).then(|| format!("probe {probe:?}: found {inside:?}"))
        })
        .collect()
}

fn collect_expected_lints(examples: &[Value]) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();
    for example in examples {
        for carrier in example
            .get("expected")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(code) = carrier.get("carrier").and_then(Value::as_str) else {
                continue;
            };
            let path = carrier.get("path").and_then(Value::as_str);
            let line = carrier
                .get("line")
                .and_then(Value::as_integer)
                .and_then(|line| u32::try_from(line).ok());
            let is_lint = !code.chars().next().is_some_and(char::is_uppercase);
            if let (Some(path), Some(line), true) = (path, line, is_lint) {
                expected.insert(format!("{path}\t{line}\t{code}"));
            }
        }
    }
    expected
}

fn build_diagnostic_key(diagnostic: &Diagnostic) -> Option<String> {
    let code = &diagnostic.code.as_ref()?.code;
    let span = diagnostic.spans.iter().find(|span| span.is_primary)?;
    let path = normalise_path(&span.file_name);
    let line = u32::try_from(span.line_start).ok()?;
    Some(format!("{path}\t{line}\t{code}"))
}

fn classify_diagnostics(
    diagnostics: &[Diagnostic],
    examples: &[Value],
    expected: &BTreeSet<String>,
) -> Vec<String> {
    let mut failures = Vec::new();
    let actual: BTreeSet<String> = diagnostics
        .iter()
        .filter_map(build_diagnostic_key)
        .collect();
    for diagnostic in diagnostics {
        let Some(key) = build_diagnostic_key(diagnostic) else {
            continue;
        };
        let span = diagnostic
            .spans
            .iter()
            .find(|span| span.is_primary)
            .expect("a keyed diagnostic has a primary span");
        let code = &diagnostic
            .code
            .as_ref()
            .expect("a keyed diagnostic has a code")
            .code;
        let path = normalise_path(&span.file_name);
        let is_probe = path.starts_with("modules/probe/");
        if is_probe || code == "dead_code" || expected.contains(&key) {
            continue;
        }
        let line = u32::try_from(span.line_start).expect("a keyed diagnostic has a u32 line");
        let fragment = find_fragment(examples, &path, line);
        let is_scaffold_import = code == "unused_imports" && fragment.is_none();
        if !is_scaffold_import {
            failures.push(format!("unexpected {diagnostic:?}"));
        }
    }
    failures.extend(
        expected
            .difference(&actual)
            .map(|carrier| format!("expected carrier did not fire: {carrier:?}")),
    );
    failures
}

fn collect_rustfmt_lines(stdout: &str, doc: &Path) -> BTreeMap<String, BTreeSet<u32>> {
    let prefix = format!(
        "{}/",
        doc.canonicalize().expect("cases/doc exists").display()
    );
    let mut changed: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    let mut path: Option<String> = None;
    let mut number = 0;
    for line in stdout.lines() {
        if let Some(header) = line.strip_prefix("Diff in ") {
            let header = header.trim_end_matches(':');
            path = header.rsplit_once(':').and_then(|(file, first)| {
                number = first.parse().ok()?;
                Some(normalise_path(file.strip_prefix(&prefix).unwrap_or(file)))
            });
            continue;
        }
        let Some(path) = &path else {
            continue;
        };
        if line.starts_with('+') {
            continue;
        }
        if line.starts_with('-') {
            changed.entry(path.clone()).or_default().insert(number);
        }
        number += 1;
    }
    changed
}

#[test]
fn given_doc_gate_manifest_when_compared_then_lint_block_and_clippy_toml_equal_the_rendered_configuration()
 {
    let rendered = render_required_configuration();
    let rules_rust = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    for root in [find_doc_gate(), rules_rust] {
        let manifest =
            fs::read_to_string(root.join("Cargo.toml")).expect("the root manifest exists");
        let clippy = fs::read_to_string(root.join("clippy.toml")).expect("clippy.toml exists");

        assert_eq!(
            (&root, manifest.contains(&rendered.workspace_lints)),
            (&root, true)
        );
        assert_eq!((&root, &rendered.clippy_toml), (&root, &clippy));
    }
}

fn check_example_structure(example: &Value, doc: &Path) -> Vec<String> {
    let claim = example.get("claim").and_then(Value::as_str).unwrap_or("?");
    let half = example.get("half").and_then(Value::as_str).unwrap_or("?");
    let label = format!("{claim} {half}");
    let mut defects = Vec::new();
    let fragments = example
        .get("fragment")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    for fragment in fragments {
        let Some(path) = fragment.get("path").and_then(Value::as_str) else {
            defects.push(format!("{label}: a fragment names no path"));
            continue;
        };
        let Ok(text) = fs::read_to_string(doc.join(path)) else {
            defects.push(format!("{label}: fragment {path} cannot be read"));
            continue;
        };
        let total = u32::try_from(text.lines().count()).unwrap_or(u32::MAX);
        let range = fragment
            .get("fixture-lines")
            .and_then(Value::as_str)
            .and_then(read_range);
        let is_inside_file = range.as_ref().is_some_and(|range| {
            let first = *range.start();
            let last = *range.end();
            (1..=last).contains(&first) && last <= total
        });
        if !is_inside_file {
            defects.push(format!(
                "{label}: fragment {path} has fixture-lines {:?}, not <first>-<last> inside 1-{total}",
                fragment.get("fixture-lines")
            ));
        }
    }
    let carriers = example
        .get("expected")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    for carrier in carriers {
        let path = carrier.get("path").and_then(Value::as_str);
        let line = carrier
            .get("line")
            .and_then(Value::as_integer)
            .and_then(|line| u32::try_from(line).ok());
        let code = carrier.get("carrier").and_then(Value::as_str);
        let (Some(path), Some(line), Some(_)) = (path, line, code) else {
            defects.push(format!(
                "{label}: expected entry {carrier} lacks a string path, an integer line or a string carrier"
            ));
            continue;
        };
        if find_fragment(std::slice::from_ref(example), path, line).is_none() {
            defects.push(format!(
                "{label}: expected {path}:{line} lies in no fragment of the example"
            ));
        }
    }
    defects
}

#[test]
fn given_doc_gate_examples_when_structure_is_checked_then_fragments_and_carriers_name_fixture_lines()
 {
    let doc = find_doc_gate();

    let defects: Vec<String> = read_examples()
        .iter()
        .flat_map(|example| check_example_structure(example, &doc))
        .collect();

    let rendered = defects.join("\n");
    assert!(defects.is_empty(), "{rendered}");
}

#[test]
fn given_broken_gate_examples_when_structure_is_checked_then_each_defect_is_reported() {
    let doc = find_doc_gate();
    let fixture = "modules/good/arch-package-rs-tree/src/lib.rs";
    let build_example = |path: &str, lines: &str, expected: &str| {
        Value::Table(
            format!(
                "claim = \"case\"\nhalf = \"good\"\nexpected = {expected}\nfragment = [{{ path = \"{path}\", fixture-lines = \"{lines}\" }}]"
            )
            .parse()
            .expect("example TOML"),
        )
    };
    let build_expected = |path: &str, line: u32| {
        format!("[{{ path = \"{path}\", line = {line}, carrier = \"AnonymousTuple\" }}]")
    };

    let missing = build_example("modules/good/missing.rs", "1-2", "[]");
    let missing_defects = check_example_structure(&missing, &doc);
    assert_eq!(missing_defects.len(), 1, "{missing_defects:?}");
    assert!(
        missing_defects[0].contains("case good"),
        "{missing_defects:?}"
    );

    for (name, example, expected_count) in [
        ("missing file", missing, 1),
        ("reversed range", build_example(fixture, "7-3", "[]"), 1),
        (
            "range beyond the file",
            build_example(fixture, "1-100000", "[]"),
            1,
        ),
        (
            "carrier below its fragment",
            build_example(fixture, "1-10", &build_expected(fixture, 11)),
            1,
        ),
        (
            "carrier in another file",
            build_example(fixture, "1-10", &build_expected("modules/good/other.rs", 1)),
            1,
        ),
        (
            "carrier on the last fragment line",
            build_example(fixture, "1-10", &build_expected(fixture, 10)),
            0,
        ),
    ] {
        let defects = check_example_structure(&example, &doc);
        assert_eq!(defects.len(), expected_count, "{name}: {defects:?}");
    }
}

#[test]
fn given_doc_gate_when_clippy_runs_then_good_halves_are_clean_bad_carriers_fire_and_probes_behave()
{
    assert_toolchain();
    let examples = read_examples();

    let diagnostics = run_clippy();

    let mut failures = check_probes(&diagnostics, &read_probes());
    failures.extend(classify_diagnostics(
        &diagnostics,
        &examples,
        &collect_expected_lints(&examples),
    ));
    let rendered = failures.join("\n");
    assert!(failures.is_empty(), "{rendered}");
}

fn classify_rustfmt_lines(
    examples: &[Value],
    changed: BTreeMap<String, BTreeSet<u32>>,
) -> Vec<String> {
    changed
        .into_iter()
        .flat_map(|(path, lines)| {
            lines
                .into_iter()
                .filter_map(|line| {
                    let is_bad = examples.iter().any(|example| {
                        example.get("half").and_then(Value::as_str) == Some("bad")
                            && find_fragment(std::slice::from_ref(example), &path, line).is_some()
                    });
                    let is_accepted = path.starts_with("modules/bad/") || is_bad;
                    (!is_accepted).then(|| format!("{path}:{line}"))
                })
                .collect::<Vec<String>>()
        })
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect()
}

#[test]
fn given_doc_gate_when_rustfmt_checks_then_no_good_example_or_scaffold_line_differs() {
    assert_toolchain();
    let doc = find_doc_gate();
    let examples = read_examples();

    let output = Command::new(find_cargo())
        .args(["fmt", "--all", "--check", "--", "--color", "never"])
        .current_dir(&doc)
        .output()
        .expect("cargo fmt runs");

    let changed = collect_rustfmt_lines(&String::from_utf8_lossy(&output.stdout), &doc);
    let failures = classify_rustfmt_lines(&examples, changed);
    assert!(failures.is_empty(), "rustfmt would change: {failures:?}");
}

#[test]
fn given_missing_probe_and_carriers_when_classified_then_drift_is_reported() {
    let diagnostics = run_clippy();
    let missing = Value::Table(
        "file=\"missing.rs\"\nitem=\"missing\""
            .parse()
            .expect("probe TOML"),
    );
    assert_eq!(check_probes(&diagnostics, &[missing]).len(), 1);
    let wrong = Value::Table(
        "file=\"probes.rs\"\nitem=\"probe_unwrap_used\"\nlint=\"never_fires\""
            .parse()
            .expect("probe TOML"),
    );
    assert_eq!(check_probes(&diagnostics, &[wrong]).len(), 1);
    assert!(!classify_diagnostics(&diagnostics, &read_examples(), &BTreeSet::new()).is_empty());
    let expected = BTreeSet::from(["missing.rs\t1\tmissing_lint".to_owned()]);
    assert_eq!(classify_diagnostics(&[], &[], &expected).len(), 1);
    let mut uncoded = diagnostics[0].clone();
    uncoded.code = None;
    assert!(classify_diagnostics(&[uncoded.clone()], &[], &BTreeSet::new()).is_empty());
    assert!(!check_probes(&[uncoded], &read_probes()).is_empty());
}

#[test]
fn given_changed_good_lines_when_checked_then_validation_rejects_them() {
    let examples = read_examples();
    let changed = BTreeMap::from([(
        "modules/good/arch-package-rs-tree/src/lib.rs".to_owned(),
        BTreeSet::from([1]),
    )]);
    assert_eq!(classify_rustfmt_lines(&examples, changed).len(), 1);
    let text = "outside header\nDiff in src/../value.rs:4:\n-old\n+new\n context\n";
    assert_eq!(
        collect_rustfmt_lines(text, &find_doc_gate()),
        BTreeMap::from([("value.rs".to_owned(), BTreeSet::from([4]))])
    );
    let malformed = Value::Table("expected=[{}]".parse().expect("TOML"));
    assert!(collect_expected_lints(&[malformed]).is_empty());
}

#[test]
fn given_malformed_gate_ranges_and_diff_headers_when_parsed_then_invalid_records_are_rejected() {
    for range in ["missing", "invalid-2", "1-invalid"] {
        assert!(read_range(range).is_none());
    }
    assert_eq!(normalise_path("./source//value.rs"), "source/value.rs");
    assert!(find_probe_range(&Value::Table(Table::new())).is_none());
    let missing = Value::Table(
        "file=\"probes.rs\"\nitem=\"no_such_item\""
            .parse()
            .expect("probe TOML"),
    );
    assert!(find_probe_range(&missing).is_none());
    assert!(
        collect_rustfmt_lines("Diff in value.rs:invalid:\n+new\n", &find_doc_gate()).is_empty()
    );
}
