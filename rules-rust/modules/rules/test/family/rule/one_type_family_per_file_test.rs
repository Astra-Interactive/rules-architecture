use crate::check::api::Rule;
use crate::check::di::CheckModule;
use crate::family::rule::OneTypeFamilyPerFile;
use crate::test::check::usecase::read_case_context;
use std::fs;

fn build_cfg_workspace() -> tempfile::TempDir {
    let context = read_case_context("OneTypeFamilyPerFile", "pass-syntax");
    let temporary = tempfile::tempdir().expect("temporary workspace");
    for file in &context.workspace.files {
        let destination = temporary.path().join(file);
        fs::create_dir_all(destination.parent().expect("file parent")).expect("fixture directory");
        fs::copy(context.workspace.root.join(file), destination).expect("copy fixture");
    }
    let wrong = temporary
        .path()
        .join("modules/sample/src/sample/model/wrong.rs");
    fs::create_dir_all(wrong.parent().expect("module parent")).expect("module directory");
    fs::write(wrong, "struct Actual;").expect("write differently named type");
    temporary
}

#[test]
fn given_inline_sibling_before_external_body_when_primary_missing_then_inline_approval_cannot_cover_it()
 {
    let temporary = build_cfg_workspace();
    fs::write(temporary.path().join("modules/sample/src/lib.rs"), "pub(crate) mod sample { pub(crate) mod model { #[cfg(windows)] #[cfg_attr(klibs_rules, expect(OneTypeFamilyPerFile, reason = \"Windows approved\"))] mod wrong {} #[cfg(unix)] mod wrong; } }").expect("write mixed declarations");
    let report = CheckModule::default()
        .check_workspace_use_case
        .invoke(temporary.path())
        .expect("valid workspace");
    assert_eq!(
        report
            .violations
            .iter()
            .filter(|finding| finding.rule_id.0 == "OneTypeFamilyPerFile")
            .count(),
        1
    );
    assert_eq!(
        report
            .violations
            .iter()
            .filter(|finding| finding.rule_id.0 == "UnfulfilledExpectation")
            .count(),
        1
    );
    assert!(report.accepted.is_empty());
}

#[test]
fn given_external_cfg_declarations_when_primary_missing_then_each_outer_approval_is_independent() {
    let temporary = build_cfg_workspace();
    for (second, missing, accepted) in [
        ("", 1, 1),
        (
            "#[cfg_attr(klibs_rules, expect(OneTypeFamilyPerFile, reason = \"Windows approved\"))]",
            0,
            2,
        ),
    ] {
        fs::write(temporary.path().join("modules/sample/src/lib.rs"), format!("pub(crate) mod sample {{ pub(crate) mod model {{ #[cfg(unix)] #[cfg_attr(klibs_rules, expect(OneTypeFamilyPerFile, reason = \"Unix approved\"))] mod wrong; #[cfg(windows)] {second} mod wrong; }} }}")).expect("write repeated declarations");
        let report = CheckModule::default()
            .check_workspace_use_case
            .invoke(temporary.path())
            .expect("valid workspace");
        assert_eq!(
            report
                .violations
                .iter()
                .filter(|finding| finding.rule_id.0 == "OneTypeFamilyPerFile")
                .count(),
            missing
        );
        assert!(
            !report
                .violations
                .iter()
                .any(|finding| finding.rule_id.0 == "UnfulfilledExpectation")
        );
        assert_eq!(report.accepted.len(), accepted);
    }
}

fn build_duplicate_options_context() -> crate::check::model::RuleContext {
    let mut context = read_case_context("OneTypeFamilyPerFile", "pass-syntax");
    context.crates[0].files[0].syntax = syn::parse_file("").expect("empty root");
    let original = &mut context.crates[0].files[1];
    original.syntax = syn::parse_file("struct Subject { options: Options } struct Options;")
        .expect("plain secondary type");
    let mut other = original.clone();
    other.path = "modules/sample/src/other/model/options.rs".to_owned();
    other.module.path = vec!["other".to_owned(), "model".to_owned(), "options".to_owned()];
    other.syntax = syn::parse_file("struct Options; impl Options { fn read(&self) {} }")
        .expect("unrelated behavioural type");
    context.crates[0].files.push(other);
    context
}

#[test]
fn given_self_and_super_paths_with_duplicate_names_when_checked_then_detects_secondary_behaviour() {
    for implementation in [
        "impl self::Options { fn read(&self) {} }",
        "mod nested { impl super::Options { fn read(&self) {} } }",
    ] {
        let mut context = build_duplicate_options_context();
        context.crates[0].files[1].syntax = syn::parse_file(&format!(
            "struct Subject {{ options: Options }} struct Options; {implementation}"
        ))
        .expect("relative implementation");
        assert_eq!(1, OneTypeFamilyPerFile.check(&context).len());
    }
}

#[test]
fn given_unknown_external_or_unsupported_self_types_when_checked_then_does_not_attach_behaviour() {
    for implementation in [
        "impl ::external::Options { fn read(&self) {} }",
        "impl <Other as Contract>::Options { fn read(&self) {} }",
        "impl (usize, usize) { fn read(&self) {} }",
        "impl crate::missing::Options { fn read(&self) {} }",
    ] {
        let mut context = build_duplicate_options_context();
        context.crates[0].files[0].syntax =
            syn::parse_file(implementation).expect("syntactic implementation");
        assert!(OneTypeFamilyPerFile.check(&context).is_empty());
    }
}

#[test]
fn given_ambiguous_kind_reexport_path_when_checked_then_does_not_choose_a_same_named_type() {
    let mut context = build_duplicate_options_context();
    let mut same_kind = context.crates[0].files[2].clone();
    same_kind.path = "modules/sample/src/sample/model/options.rs".to_owned();
    same_kind.module.path = vec![
        "sample".to_owned(),
        "model".to_owned(),
        "options".to_owned(),
    ];
    context.crates[0].files.push(same_kind);
    context.crates[0].files[0].syntax =
        syn::parse_file("impl crate::sample::model::Options { fn read(&self) {} }")
            .expect("ambiguous kind reexport");
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_unrelated_same_named_type_when_checked_then_plain_secondary_type_stays_in_its_family() {
    let context = build_duplicate_options_context();
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_nested_same_named_type_when_checked_then_its_behaviour_does_not_escape_its_scope() {
    let mut context = build_duplicate_options_context();
    context.crates[0].files[1].syntax = syn::parse_file("struct Subject { options: Options } struct Options; mod nested { struct Options; impl Options { fn read(&self) {} } }").expect("nested shadowing type");
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_local_secondary_behaviour_with_duplicate_names_when_checked_then_reports_the_actual_owner()
{
    let mut context = build_duplicate_options_context();
    context.crates[0].files[1].syntax = syn::parse_file(
        "struct Subject { options: Options } struct Options; impl Options { fn read(&self) {} }",
    )
    .expect("local implementation");
    let findings = OneTypeFamilyPerFile.check(&context);
    assert_eq!(1, findings.len());
    assert!(findings[0].finding.subject.ends_with("subject::Options"));
}

#[test]
fn given_ambiguous_bare_cross_file_impl_when_checked_then_does_not_guess_the_owner() {
    let mut context = build_duplicate_options_context();
    context.crates[0].files[0].syntax =
        syn::parse_file("impl Options { fn read(&self) {} }").expect("unresolved imported type");
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_explicit_lexical_paths_when_checked_then_resolves_only_the_matching_owner() {
    for implementation in [
        "impl crate::sample::model::subject::Options { fn read(&self) {} }",
        "mod extra { impl super::sample::model::subject::Options { fn read(&self) {} } }",
    ] {
        let mut context = build_duplicate_options_context();
        context.crates[0].files[0].syntax =
            syn::parse_file(implementation).expect("lexically qualified impl");
        assert_eq!(1, OneTypeFamilyPerFile.check(&context).len());
    }
    let mut context = build_duplicate_options_context();
    context.crates[0].files[0].syntax =
        syn::parse_file("impl crate::other::model::Options { fn read(&self) {} }")
            .expect("kind reexport path");
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_secondary_type_behaviour_in_another_module_when_checked_then_reports_its_family() {
    let mut context = read_case_context("OneTypeFamilyPerFile", "pass-syntax");
    context.crates[0].files[0].syntax =
        syn::parse_file("mod extra { impl Child { fn read(&self) {} } }").expect("valid source");
    assert_eq!(1, OneTypeFamilyPerFile.check(&context).len());
}

#[test]
fn given_target_file_suffix_when_checked_then_matches_the_unsuffixed_type() {
    let mut context = read_case_context("OneTypeFamilyPerFile", "pass-syntax");
    let file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| !file.module.path.is_empty())
        .expect("source file");
    file.path = "modules/sample/src/sample/model/subject_linux.rs".to_owned();
    let id = file.module.clone();
    let module = context.crates[0]
        .modules
        .iter_mut()
        .find(|module| module.id == id)
        .expect("file module");
    module
        .declaration
        .as_mut()
        .expect("declaration")
        .target_value = Some("linux".to_owned());
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_plain_function_files_when_checked_then_requires_no_primary_type() {
    crate::test::syntax::util::assert_syntax_findings(
        &OneTypeFamilyPerFile,
        "OneTypeFamilyPerFile",
        "fn run() {}",
        0,
    );
}

#[test]
fn given_union_and_grouped_payload_types_when_checked_then_follows_field_type_uses() {
    crate::test::syntax::util::assert_syntax_findings(
        &OneTypeFamilyPerFile,
        "OneTypeFamilyPerFile",
        "union Subject { payload: Payload, array: [Extra; 2] } struct Payload { nested: (Leaf) } struct Extra; struct Leaf;",
        0,
    );
}

#[test]
fn given_factory_trait_and_payload_family_when_checked_then_keeps_required_secondary_types() {
    crate::test::syntax::util::assert_syntax_findings(
        &OneTypeFamilyPerFile,
        "OneTypeFamilyPerFile",
        "pub(crate) enum Subject { Data(Payload) } struct Payload { child: Child } struct Child; trait SubjectFactory { fn invoke(&self) -> Subject; }",
        0,
    );
}

#[test]
fn given_secondary_behaviour_unrelated_traits_and_functions_when_checked_then_reports_each() {
    crate::test::syntax::util::assert_syntax_findings(
        &OneTypeFamilyPerFile,
        "OneTypeFamilyPerFile",
        "struct Payload; struct Subject { payload: Payload, extra: Extra } struct Extra; impl Extra { fn read(&self) {} } trait Other {} fn run() {}",
        4,
    );
}

#[test]
fn given_wrong_primary_filename_when_checked_then_reports_module_anchor() {
    let mut context = read_case_context("OneTypeFamilyPerFile", "pass-syntax");
    let file = context.crates[0]
        .files
        .iter_mut()
        .find(|file| !file.module.path.is_empty())
        .expect("source file");
    file.syntax = syn::parse_file("struct Unrelated;").expect("valid input");
    let findings = OneTypeFamilyPerFile.check(&context);
    assert_eq!(1, findings.len());
    assert!(matches!(
        findings[0].anchor,
        crate::check::model::Anchor::Module { .. }
    ));
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("OneTypeFamilyPerFile", "fail-syntax");
    assert!(!OneTypeFamilyPerFile.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("OneTypeFamilyPerFile", "pass-syntax");
    assert!(OneTypeFamilyPerFile.check(&context).is_empty());
}
