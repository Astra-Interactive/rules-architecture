use crate::check::api::Rule;
use crate::naming::rule::ExplicitItName;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_all_binding_positions_when_checked_then_reports_it() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExplicitItName,
        "ExplicitItName",
        "struct Row { it: usize } trait Reader { fn read(it: usize); } impl Reader for Row { fn read(it: usize) { let it = 0; if let Some(it) = None {} while let Some(it) = None {} for it in [] {} match None { Some(it) => {}, _ => {} } 'it: loop { break 'it; } let _ = |it| it; } }",
        10,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ExplicitItName", "fail-syntax");
    assert!(!ExplicitItName.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ExplicitItName", "pass-syntax");
    assert!(ExplicitItName.check(&context).is_empty());
}
