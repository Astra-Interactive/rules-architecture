use crate::check::api::Rule;
use crate::module::rule::ReExportShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::assert_resolution_cases;
use crate::test::source::syn::build_index_files;
use std::collections::BTreeMap;

#[test]
fn given_persistent_resolution_cases_when_checked_then_missing_export_is_the_only_rule_finding() {
    assert_resolution_cases(
        &ReExportShape,
        "ReExportShape",
        "modules/sample/src/model/value.rs:1\tValue\n",
    );
}

#[test]
fn given_file_module_exports_when_checked_then_named_export_is_required_and_private_export_is_rejected()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index_files(
        "mod model { pub(crate) use value::Good; pub(crate) use value::Secret; pub(crate) use value::Renamed as Alias; mod value; }",
        BTreeMap::from([(
            "model/value.rs",
            "pub(crate) struct Good; pub(crate) struct Missing; pub(crate) struct Renamed; struct Secret;",
        )]),
    )];
    let findings = ReExportShape.check(&context);
    assert_eq!(findings.len(), 3);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.subject != "Good")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject == "Secret"
                && finding.finding.location.path == "lib.rs")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.finding.subject == "Missing"
                && finding.finding.location.path == "model/value.rs")
    );
}

#[test]
fn given_duplicate_and_glob_exports_when_checked_then_each_non_private_item_needs_one_exact_export()
{
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index_files(
        "mod model { pub(crate) use value::Good; pub(crate) use value::Good; pub(crate) use value::*; mod value; }",
        BTreeMap::from([(
            "model/value.rs",
            "pub(crate) struct Good; pub(crate) struct OnlyGlob;",
        )]),
    )];
    assert_eq!(ReExportShape.check(&context).len(), 2);
}

#[test]
fn given_complementary_cfg_exports_when_checked_then_exactly_one_export_is_required_in_each_configuration()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index_files(
        "mod model { #[cfg(feature = \"left\")] pub(crate) use value::Value; #[cfg(not(feature = \"left\"))] pub(crate) use value::Value; mod value; }",
        BTreeMap::from([("model/value.rs", "pub(crate) struct Value;")]),
    )];
    assert!(ReExportShape.check(&context).is_empty());
    context.indexes = vec![build_index_files(
        "mod model { #[cfg(feature = \"left\")] pub(crate) use value::Value; mod value; }",
        BTreeMap::from([("model/value.rs", "pub(crate) struct Value;")]),
    )];
    assert_eq!(ReExportShape.check(&context).len(), 1);
}
