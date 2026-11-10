use crate::check::api::Rule;
use crate::check::model::RuleFinding;
use crate::config::model::RequiredConfiguration;
use crate::config::rule::LintConfig;
use crate::config::util::render_required_configuration;
use crate::test::check::usecase::read_case_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::model::{RustfmtFacts, ToolchainFile};
use crate::workspace::util::parse_toml_file;
use std::collections::BTreeSet;
use std::fs;

#[test]
fn given_warn_groups_and_compiler_flags_when_checked_then_they_do_not_lower_lints() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    let required = render_required_configuration();
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        &format!(
            "[workspace]\nresolver=\"3\"\n{}\nall = {{ level = \"warn\", priority = -1 }}\n",
            required.workspace_lints
        ),
    )
    .expect("required lints with warn group");
    context.workspace.cargo_config = Some(
        parse_toml_file(
            ".cargo/config.toml",
            "[build]\nrustflags=[\"-C\",\"debuginfo=1\"]\n",
        )
        .expect("compiler flags"),
    );
    assert_eq!(LintConfig::new(required).check(&context), Vec::new());
}

#[test]
fn given_missing_formatter_and_nightly_manifest_keys_when_checked_then_each_is_reported() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.rustfmt = RustfmtFacts::Missing {
        binary: "missing-rustfmt".to_owned(),
    };
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "cargo-features = [\"unstable\"]\n[workspace]\nresolver = \"3\"\n",
    )
    .expect("syntax-only manifest");
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject == "rustfmt")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject == "Cargo.toml cargo-features")
    );
    context.workspace.rustfmt_files =
        vec![parse_toml_file(".rustfmt.toml", "").expect("formatter configuration")];
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject == "rustfmt"
                && finding.finding.location.path == ".rustfmt.toml")
    );
}

#[test]
fn given_invalid_required_templates_when_checked_then_optional_checks_still_run() {
    let context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    let required = RequiredConfiguration {
        workspace_lints: "[".to_owned(),
        clippy_toml: "[".to_owned(),
    };
    let findings = LintConfig::new(required).check(&context);
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.location.path == "required configuration")
    );
}

#[test]
fn given_noninteger_priority_and_channelless_toolchain_when_checked_then_checks_remain_defined() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.workspace.root_manifest = parse_toml_file(
        "Cargo.toml",
        "[workspace.lints.rust]\nunused_must_use = { level = \"warn\", priority = \"invalid\" }\n",
    )
    .expect("TOML syntax");
    context.workspace.toolchain_file = Some(ToolchainFile::Toml(
        parse_toml_file("rust-toolchain.toml", "[toolchain]\ncomponents=[]\n")
            .expect("TOML syntax"),
    ));
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject.contains("unreachable_pub"))
    );
}

fn assert_tdd_lint_config_invalid_chrono_now_allow_invalid(value: &str) {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    let required = render_required_configuration();
    let required_entry = "{ path = \"chrono::Utc::now\", reason = \"time-api.rs.md: the moment comes from the Clock\", allow-invalid = true },";
    let actual_entry = format!(
        "{{ path = \"chrono::Utc::now\", reason = \"time-api.rs.md: the moment comes from the Clock\"{value} }},"
    );
    let actual_clippy = required.clippy_toml.replace(required_entry, &actual_entry);
    assert_ne!(actual_clippy, required.clippy_toml);
    context.workspace.clippy_files = vec![
        parse_toml_file(".clippy.toml", &actual_clippy).expect("mutated Clippy configuration"),
    ];

    let findings = LintConfig::new(required).check(&context);
    assert_eq!(1, findings.len());
    assert_eq!("LintConfig", findings[0].finding.rule_id.0);
    assert_eq!(".clippy.toml", findings[0].finding.location.path);
    assert_eq!(Some(21), findings[0].finding.location.line);
}

#[test]
fn given_tdd_lint_config_missing_chrono_allow_invalid_when_checked_then_reports_config_line() {
    assert_tdd_lint_config_invalid_chrono_now_allow_invalid("");
}

#[test]
fn given_tdd_lint_config_false_chrono_allow_invalid_when_checked_then_reports_config_line() {
    assert_tdd_lint_config_invalid_chrono_now_allow_invalid(", allow-invalid = false");
}

#[test]
fn given_tdd_lint_config_string_chrono_allow_invalid_when_checked_then_reports_config_line() {
    assert_tdd_lint_config_invalid_chrono_now_allow_invalid(", allow-invalid = \"true\"");
}

fn list_findings_at(findings: &[RuleFinding], path: &str, line: u32) -> Vec<RuleFinding> {
    findings
        .iter()
        .filter(|finding| {
            finding.finding.location.path == path && finding.finding.location.line == Some(line)
        })
        .cloned()
        .collect()
}

#[test]
fn given_resolver_two_with_cargo_fallback_when_checked_then_resolver_three_is_still_required() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.workspace.root_manifest =
        parse_toml_file("Cargo.toml", "[workspace]\nresolver = \"2\"\n").expect("workspace");
    context.workspace.cargo_config = Some(
        parse_toml_file(
            ".cargo/config.toml",
            "[resolver]\nincompatible-rust-versions = \"fallback\"\n",
        )
        .expect("cargo configuration"),
    );
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    let resolver = list_findings_at(&findings, "Cargo.toml", 2);
    assert_eq!(1, resolver.len(), "findings: {findings:?}");
    assert_eq!("LintConfig", resolver[0].finding.rule_id.0);
}

#[test]
fn given_dated_stable_channel_and_nightly_channel_when_checked_then_only_nightly_is_reported() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.workspace.toolchain_file = Some(ToolchainFile::Toml(
        parse_toml_file(
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"stable-2026-01-22\"\n",
        )
        .expect("toolchain"),
    ));
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.location.path != "rust-toolchain.toml"),
        "findings: {findings:?}"
    );
    context.workspace.toolchain_file = Some(ToolchainFile::Toml(
        parse_toml_file(
            "rust-toolchain.toml",
            "[toolchain]\nchannel = \"nightly-2026-01-22\"\n",
        )
        .expect("toolchain"),
    ));
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    let channel = list_findings_at(&findings, "rust-toolchain.toml", 2);
    assert_eq!(1, channel.len(), "findings: {findings:?}");
    assert_eq!("LintConfig", channel[0].finding.rule_id.0);
}

#[test]
fn given_zero_nesting_threshold_when_checked_then_reports_the_key_line() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    let required = render_required_configuration();
    let actual_clippy = required.clippy_toml.replace(
        "excessive-nesting-threshold = 5",
        "excessive-nesting-threshold = 0",
    );
    assert_ne!(actual_clippy, required.clippy_toml);
    context.workspace.clippy_files =
        vec![parse_toml_file(".clippy.toml", &actual_clippy).expect("Clippy configuration")];
    let findings = LintConfig::new(required).check(&context);
    assert_eq!(1, findings.len(), "findings: {findings:?}");
    assert_eq!("LintConfig", findings[0].finding.rule_id.0);
    assert_eq!(".clippy.toml", findings[0].finding.location.path);
    assert_eq!(Some(4), findings[0].finding.location.line);
}

#[test]
fn given_nightly_rustfmt_key_in_member_and_root_when_checked_then_each_file_is_reported() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let member = directory.path().join("modules/billing/api");
    fs::create_dir_all(member.join("src")).expect("member source directory");
    fs::write(
        directory.path().join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"modules/billing/api\"]\n",
    )
    .expect("workspace manifest");
    fs::write(
        member.join("Cargo.toml"),
        "[package]\nname = \"shop-billing-api\"\nedition = \"2024\"\n",
    )
    .expect("member manifest");
    fs::write(member.join("src/lib.rs"), "").expect("member source");
    fs::write(
        member.join("rustfmt.toml"),
        "group_imports = \"StdExternalCrate\"\n",
    )
    .expect("member formatter configuration");
    fs::write(
        directory.path().join("rustfmt.toml"),
        "group_imports = \"StdExternalCrate\"\n",
    )
    .expect("root formatter configuration");
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.workspace = CargoWorkspaceReader::new(crate::workspace::util::resolve_cargo_path())
        .read_workspace(directory.path())
        .expect("read workspace");
    context.rustfmt = RustfmtFacts::Queried {
        nightly_only_keys: BTreeSet::from(["group_imports".to_owned()]),
    };
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    let mut paths: Vec<String> = findings
        .iter()
        .filter(|finding| finding.finding.subject.ends_with("group_imports"))
        .map(|finding| {
            assert_eq!("LintConfig", finding.finding.rule_id.0);
            assert_eq!(Some(1), finding.finding.location.line);
            finding.finding.location.path.clone()
        })
        .collect();
    paths.sort();
    assert_eq!(
        vec!["modules/billing/api/rustfmt.toml", "rustfmt.toml"],
        paths,
        "findings: {findings:?}"
    );
}

#[test]
fn given_group_lowering_required_lints_when_checked_then_the_group_line_is_reported() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    let required = render_required_configuration();
    let manifest = format!(
        "[workspace]\nresolver=\"3\"\n{}\nrestriction = {{ level = \"allow\", priority = 1 }}\n",
        required.workspace_lints
    );
    let line = manifest
        .lines()
        .position(|text| text.starts_with("restriction"))
        .and_then(|index| u32::try_from(index + 1).ok())
        .expect("restriction line");
    context.workspace.root_manifest =
        parse_toml_file("Cargo.toml", &manifest).expect("lints with a restriction group");
    let findings = LintConfig::new(required).check(&context);
    let group = list_findings_at(&findings, "Cargo.toml", line);
    assert!(!group.is_empty(), "findings: {findings:?}");
    assert!(
        group
            .iter()
            .all(|finding| finding.finding.rule_id.0 == "LintConfig")
    );
}

#[test]
fn given_target_rustflags_setting_the_klibs_cfg_when_checked_then_reports_the_flags_line() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.workspace.cargo_config = Some(
        parse_toml_file(
            ".cargo/config.toml",
            "[target.'cfg(all())']\nrustflags = [\"--cfg\", \"klibs_rules\"]\n",
        )
        .expect("target rustflags"),
    );
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    let flags = list_findings_at(&findings, ".cargo/config.toml", 2);
    assert_eq!(1, flags.len(), "findings: {findings:?}");
    assert_eq!("LintConfig", flags[0].finding.rule_id.0);
}

#[test]
fn given_rustc_bootstrap_in_cargo_env_when_checked_then_reports_every_value_but_minus_one() {
    for entry in [
        "RUSTC_BOOTSTRAP = \"1\"",
        "RUSTC_BOOTSTRAP = { value = \"1\", force = true }",
    ] {
        let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
        context.workspace.cargo_config = Some(
            parse_toml_file(".cargo/config.toml", &format!("[env]\n{entry}\n"))
                .expect("cargo environment"),
        );
        let findings = LintConfig::new(render_required_configuration()).check(&context);
        let bootstrap = list_findings_at(&findings, ".cargo/config.toml", 2);
        assert_eq!(1, bootstrap.len(), "{entry}: {findings:?}");
        assert_eq!("LintConfig", bootstrap[0].finding.rule_id.0);
    }
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    context.workspace.cargo_config = Some(
        parse_toml_file(
            ".cargo/config.toml",
            "[env]\nRUSTC_BOOTSTRAP = \"-1\"\nRUST_LOG = \"info\"\n",
        )
        .expect("cargo environment"),
    );
    let findings = LintConfig::new(render_required_configuration()).check(&context);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.location.path != ".cargo/config.toml"),
        "findings: {findings:?}"
    );
}

#[test]
fn given_std_lock_disallowed_types_without_reason_when_checked_then_reports_each_entry_line() {
    let mut context = read_case_context("LintConfig", "pass-dot-clippy-toml");
    let required = render_required_configuration();
    let reason = "reason = \"async.rs.md, Shared state and locks: std locks poison\"";
    let actual_clippy = required
        .clippy_toml
        .replace(
            &format!("{{ path = \"std::sync::Mutex\", {reason} }},"),
            "\"std::sync::Mutex\",",
        )
        .replace(
            &format!("{{ path = \"std::sync::RwLock\", {reason} }},"),
            "{ path = \"std::sync::RwLock\", reason = \" \" },",
        )
        .replace(
            &format!("{{ path = \"std::sync::PoisonError\", {reason} }},"),
            "{ path = \"std::sync::PoisonError\" },",
        );
    assert_ne!(actual_clippy, required.clippy_toml);
    context.workspace.clippy_files =
        vec![parse_toml_file(".clippy.toml", &actual_clippy).expect("Clippy configuration")];
    let findings = LintConfig::new(required).check(&context);
    let mut lines: Vec<u32> = findings
        .iter()
        .map(|finding| {
            assert_eq!("LintConfig", finding.finding.rule_id.0);
            assert_eq!(".clippy.toml", finding.finding.location.path);
            finding.finding.location.line.expect("entry line")
        })
        .collect();
    lines.sort_unstable();
    assert_eq!(vec![10, 11, 12], lines, "findings: {findings:?}");
    let required = render_required_configuration();
    let reworded = required
        .clippy_toml
        .replace(reason, "reason = \"poisoning is not wanted\"");
    assert_ne!(reworded, required.clippy_toml);
    context.workspace.clippy_files =
        vec![parse_toml_file(".clippy.toml", &reworded).expect("Clippy configuration")];
    assert_eq!(LintConfig::new(required).check(&context), Vec::new());
}
