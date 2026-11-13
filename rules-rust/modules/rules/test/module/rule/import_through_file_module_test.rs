use crate::check::api::Rule;
use crate::module::rule::ImportThroughFileModule;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::assert_resolution_cases;
use crate::test::source::syn::build_index_files;
use std::collections::BTreeMap;

#[test]
fn given_persistent_resolution_cases_when_checked_then_file_module_bypass_is_the_only_rule_finding()
{
    assert_resolution_cases(
        &ImportThroughFileModule,
        "ImportThroughFileModule",
        "modules/sample/src/util/value_ext.rs:1\tcrate::model::value::Value\n",
    );
}

#[test]
fn given_file_module_paths_when_checked_then_kind_exports_are_allowed_and_bypasses_are_reported_at_real_sites()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.indexes = vec![build_index_files(
        "mod model { pub(crate) use value::Value; mod value; }\nmod util { use crate::model::value::Value; fn call() { crate::model::Value; crate::model::value::Value; } }",
        BTreeMap::from([("model/value.rs", "pub(crate) struct Value;")]),
    )];
    let findings = ImportThroughFileModule.check(&context);
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.location.path == "lib.rs"
                && finding.finding.location.line == Some(2))
    );
}
