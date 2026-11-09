use crate::check::api::Rule;
use crate::family::rule::ConstantPlacement;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_constant_holder_types_and_modules_when_checked_then_reports_them_beside_a_type() {
    crate::test::syntax::util::assert_syntax_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        "struct Subject; struct Defaults; impl Defaults { const VALUE: usize = 1; } enum Keys {} impl Keys { const KEY: usize = 1; } mod limits { const LIMIT: usize = 1; }",
        3,
    );
}

#[test]
fn given_trait_constants_statics_or_behaviour_when_checked_then_keeps_valid_items() {
    crate::test::syntax::util::assert_syntax_findings(
        &ConstantPlacement,
        "ConstantOutsideImpl",
        "const VALUE: usize = 1; static COUNT: usize = 1; trait Subject {} struct Other; impl Other { fn read(&self) {} } mod mixed { const VALUE: usize = 1; fn run() {} } mod empty {} mod external;",
        0,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ConstantOutsideImpl", "fail-syntax");
    assert!(!ConstantPlacement.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ConstantOutsideImpl", "pass-syntax");
    assert!(ConstantPlacement.check(&context).is_empty());
}
