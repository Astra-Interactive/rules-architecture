use crate::check::api::Rule;
use crate::module::rule::ImplementationVisibility;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_trait_reexport_and_cfg_alternatives_when_checked_then_only_matching_technology_impl_reports()
 {
    let mut context = read_shape_context(
        "mod api { pub trait Repository {} } mod bridge { pub use crate::api::Repository as Alias; } mod file { #[cfg(unix)] pub struct Store; #[cfg(unix)] impl crate::bridge::Alias for Store {} #[cfg(not(unix))] pub struct OtherStore; #[cfg(unix)] impl crate::bridge::Alias for OtherStore {} } mod memory { pub struct Cache; impl crate::bridge::Alias for Cache {} }",
        &[],
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    let findings = ImplementationVisibility.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case_sample::file::Store");
}

#[test]
fn given_public_enum_implementing_a_crate_trait_in_technology_kind_when_checked_then_it_is_reported()
 {
    let mut context = read_shape_context(
        "mod api { pub trait Repository {} }\nmod file {\n    pub enum Store { Open, Closed } impl crate::api::Repository for Store {}\n    pub(crate) enum Cache { Warm } impl crate::api::Repository for Cache {}\n    pub enum Mode { Fast }\n}",
        &[],
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"\n",
    )
    .unwrap();
    let findings = ImplementationVisibility.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "case_sample::file::Store");
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("ImplementationVisibility", "fail-declaration-shape");
    let mut actual: Vec<String> = ImplementationVisibility
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
        [
            "modules/sample/src/file/file_invoice_repository.rs:2 case_sample::file::FileInvoiceRepository"
        ]
    );
    let context = read_case_context("ImplementationVisibility", "pass-declaration-shape");
    assert!(ImplementationVisibility.check(&context).is_empty());
}
