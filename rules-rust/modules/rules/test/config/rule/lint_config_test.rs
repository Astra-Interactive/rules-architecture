use crate::check::api::Rule;
use crate::config::model::RequiredConfiguration;
use crate::config::rule::LintConfig;
use crate::config::util::render_required_configuration;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{RustfmtFacts, ToolchainFile};
use crate::workspace::util::parse_toml_file;

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
