use crate::check::api::Rule;
use crate::import::rule::ImportOrdering;
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_findings;

#[test]
fn given_root_module_tree_imports_when_checked_then_leaves_block_order_to_root_file_shape() {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.crates[0].files[0].syntax =
        syn::parse_file("mod sample { mod nested; use nested::Value; }").expect("valid source");
    assert!(ImportOrdering.check(&context).is_empty());
}

#[test]
fn given_blank_line_between_imports_of_a_root_module_block_when_checked_then_reports_the_import() {
    let source = "pub mod model {\n    pub use invoice::Invoice;\n\n    pub use invoice_error::InvoiceError;\n    mod invoice;\n    mod invoice_error;\n}\npub mod api {\n    // comment above the first use\n    pub use contract::Contract;\n    #[cfg(unix)]\n    pub use unix_contract::UnixContract;\n    #[cfg(windows)]\n    pub use windows_contract::WindowsContract;\n\n    mod contract;\n}\npub mod spec {\n    pub use first::First;\n    // reason\n    pub use second::Second;\n    mod first;\n    mod second;\n}";
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    context.crates[0].files[0].syntax = syn::parse_file(source).expect("valid source");
    context.crates[0].files[0].comments = crate::source::util::scan_comments(source, &[]);
    let lines: Vec<_> = ImportOrdering
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(4), Some(21)], lines);
}

#[test]
fn given_non_private_use_after_an_item_in_a_file_module_when_checked_then_reports_the_import() {
    for source in [
        "fn run() {}\npub use std::fmt::Debug;",
        "struct Value;\npub(crate) use std::fmt::Debug;",
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
fn given_non_private_use_after_an_item_at_the_crate_root_when_checked_then_leaves_it_to_re_export_shape()
 {
    let mut context = read_case_context("ImportOrdering", "pass-syntax");
    assert!(context.crates[0].files[0].module.path.is_empty());
    context.crates[0].files[0].syntax =
        syn::parse_file("fn run() {}\npub use std::fmt::Debug;").expect("valid source");
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
fn given_trailing_comment_on_last_import_when_checked_then_reports_that_import() {
    assert_code_findings(
        &ImportOrdering,
        "ImportOrdering",
        "use std::collections::HashMap;\nuse std::fmt::Display; // rendered in the message\n\nstruct Names {\n    names: HashMap<u32, String>,\n}\nfn describe<T: Display>(value: T) {}\n",
        &[2],
    );
    assert_code_findings(
        &ImportOrdering,
        "ImportOrdering",
        "use std::fmt::Display; // rendered in the message\n\nfn describe<T: Display>(value: T) {}\n",
        &[1],
    );
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
fn given_blank_line_or_comment_inside_the_imports_of_a_function_body_when_checked_then_reports_the_import()
 {
    let source = "fn spaced() {\n    use std::fmt::Debug;\n\n    use std::fmt::Display;\n    let value = 1;\n}\nfn commented() {\n    use std::fmt::Debug;\n    // reason\n    use std::fmt::Display;\n    let value = 1;\n}\nfn trailing() {\n    use std::fmt::Debug;\n    use std::fmt::Display; // rendered in the message\n    let value = 1;\n}\nfn conforming() {\n    // comment above the block\n    use std::fmt::Debug;\n    use std::fmt::Display;\n\n    let value = 1;\n}";
    assert_code_findings(&ImportOrdering, "ImportOrdering", source, &[4, 10, 15]);
}

#[test]
fn given_comment_after_a_block_closing_on_the_use_line_when_checked_then_reports_nothing() {
    for source in [
        "fn f() {\n    let value = { use std::fmt::Write; make() }; // why\n}",
        "fn g(items: Vec<u8>) {\n    items.iter().map(|v| { use a::T; v.t() }) // why\n}",
    ] {
        assert_code_findings(&ImportOrdering, "ImportOrdering", source, &[]);
    }
}

#[test]
fn given_last_body_import_after_blank_line_with_trailing_comment_when_checked_then_reports_once() {
    assert_code_findings(
        &ImportOrdering,
        "ImportOrdering",
        "fn f() {\n    use std::fmt::Debug;\n\n    use std::fmt::Display; // why\n    let value = 1;\n}",
        &[4],
    );
}

#[test]
fn given_trailing_comment_on_a_non_last_body_import_when_checked_then_reports_the_next_import() {
    assert_code_findings(
        &ImportOrdering,
        "ImportOrdering",
        "fn f() {\n    use std::fmt::Debug; // why\n    use std::fmt::Display;\n    let value = 1;\n}",
        &[3],
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
