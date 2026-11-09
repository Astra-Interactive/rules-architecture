use crate::check::api::Rule;
use crate::ordering::rule::DeclarationOrder;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_incomplete_families_and_late_file_items_when_checked_then_reports_positions() {
    crate::test::syntax::util::assert_syntax_findings(
        &DeclarationOrder,
        "DeclarationOrder",
        "impl Subject {} struct Subject; impl Reader for Subject { fn read() {} const VALUE: usize = 1; type Value = usize; } struct Child; impl Subject {} fn run() {} struct Last; static VALUE: usize = 1;",
        7,
    );
}

#[test]
fn given_external_inherent_impl_and_trait_requirements_when_checked_then_keeps_valid_order() {
    crate::test::syntax::util::assert_syntax_findings(
        &DeclarationOrder,
        "DeclarationOrder",
        "type Task = usize; impl External { fn open() -> Option<External> { None } fn read(&self) {} const VALUE: usize = 1; } impl Reader for External { type Value = usize; const VALUE: usize = 1; fn read() {} }",
        0,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("DeclarationOrder", "fail-syntax");
    assert!(!DeclarationOrder.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("DeclarationOrder", "pass-syntax");
    assert!(DeclarationOrder.check(&context).is_empty());
}
