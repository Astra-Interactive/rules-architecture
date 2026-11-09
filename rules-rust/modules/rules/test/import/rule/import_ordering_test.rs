use crate::check::api::Rule;
use crate::import::rule::ImportOrdering;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_root_module_tree_imports_when_checked_then_leaves_block_order_to_root_file_shape() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.crates[0].files[0].syntax =
        syn::parse_file("mod sample { mod nested; use nested::Value; }").expect("valid source");
    assert!(ImportOrdering.check(&context).is_empty());
}

#[test]
fn given_comments_between_imports_when_checked_then_reports_the_interrupted_block() {
    for source in [
        "use std::fmt::Debug; // reason\nuse std::fmt::Display;",
        "use std::fmt::Debug; /* reason */ use std::fmt::Display;",
        "use std::fmt::Debug;\n// reason\nuse std::fmt::Display;",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &ImportOrdering,
            "ImportOrdering",
            source,
            1,
        );
    }
}

#[test]
fn given_scoped_late_imports_and_file_gaps_when_checked_then_reports_each() {
    crate::test::syntax::util::assert_syntax_findings(
        &ImportOrdering,
        "ImportOrdering",
        "use std::fmt::Debug;\n\nuse std::fmt::Display;\nmod nested { fn run() {} use std::fmt::Write; }\nfn run() { let value = 1; use std::fmt::Debug; }",
        3,
    );
}

#[test]
fn given_leading_comments_and_scope_imports_when_checked_then_keeps_them() {
    crate::test::syntax::util::assert_syntax_findings(
        &ImportOrdering,
        "ImportOrdering",
        "// path comment\nuse std::fmt::Debug;\nuse std::fmt::Display;\nmod nested { use std::fmt::Write; fn run() {} }\nfn run() { use std::fmt::Debug; let value = 1; }",
        0,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ImportOrdering", "fail-syntax");
    assert!(!ImportOrdering.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ImportOrdering", "pass-syntax");
    assert!(ImportOrdering.check(&context).is_empty());
}
