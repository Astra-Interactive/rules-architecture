use crate::check::api::Rule;
use crate::module::rule::PluralPackageName;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_project_plural_words_when_checked_then_configured_words_pass() {
    let mut context = read_shape_context(
        "mod invoices {} mod players {} mod news {} mod status {} mod analysis {} mod class {}",
        &[],
    );
    assert_eq!(PluralPackageName.check(&context).len(), 2);
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nplural-concepts=['invoices','players']",
    )
    .expect("manifest");
    assert!(PluralPackageName.check(&context).is_empty());
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("PluralPackageName", "fail-declaration-shape");
    let mut actual: Vec<String> = PluralPackageName
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        ["modules/sample/src/lib.rs:1 case_sample::invoices"]
    );
    let context = read_case_context("PluralPackageName", "pass-declaration-shape");
    assert!(PluralPackageName.check(&context).is_empty());
}
