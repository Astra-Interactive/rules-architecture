use crate::check::api::Rule;
use crate::import::rule::ImportOrdering;
use crate::import::rule::WildcardImport;
use crate::module::rule::{PlatformPackage, TestMirror};
use crate::naming::rule::{ClosureParameterName, ExplicitItName};
use crate::suppression::rule::ExpectScope;
use crate::test::check::usecase::read_case_context;
use crate::test::syntax::util::assert_syntax_findings;
use crate::tuple::rule::AnonymousTuple;

#[test]
fn given_expression_list_macros_when_checked_then_names_and_tuple_annotations_are_visited_recursively()
 {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[1].syntax = syn::parse_file("fn read() { vec![|x| x, |it| it, |value: (u8, u8)| value]; outer!(inner!(|y| y)); opaque!(type => |z| z); }").expect("valid syntax");
    assert_eq!(ClosureParameterName.check(&context).len(), 2);
    assert_eq!(ExplicitItName.check(&context).len(), 1);
    assert_eq!(AnonymousTuple.check(&context).len(), 1);
}

#[test]
fn given_blocks_inside_macro_arguments_when_checked_then_import_and_expectation_rules_keep_their_scope()
 {
    assert_syntax_findings(
        &WildcardImport,
        "ImportOrdering",
        "fn read() { vec![{ use module::*; 0 }]; opaque!(use module::*); }",
        1,
    );
    assert_syntax_findings(
        &ImportOrdering,
        "ImportOrdering",
        "fn read() { vec![{ let value = 0; use std::fmt::Debug; value }]; }",
        1,
    );
    assert_syntax_findings(
        &ExpectScope,
        "ExpectScope",
        "fn read() { vec![{ #[expect(clippy::clone_on_copy, reason = \"test case\")] let value = 0; value }]; }",
        1,
    );
    assert_syntax_findings(
        &TestMirror,
        "ImportOrdering",
        "fn read() { vec![{ #[test] fn check() {} }]; }",
        1,
    );
}

#[test]
fn given_platform_attribute_inside_macro_argument_when_checked_then_target_modules_are_reserved() {
    let mut context = read_case_context("PlatformPackage", "pass-concept");
    context.crates[0].files[0].syntax =
        syn::parse_file("fn read() { vec![{ #[cfg(target_arch = \"arm\")] struct Value; }]; }")
            .expect("valid syntax");
    assert_eq!(PlatformPackage.check(&context).len(), 1);
}
