use crate::check::api::Rule;
use crate::ordering::rule::CalleeBeforeCaller;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_qualified_self_calls_when_checked_then_resolves_only_the_current_type() {
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "impl Subject { fn first(&self) { <Subject>::second(); <Subject as Other>::second(); } fn second() {} fn third() { <Other>::fourth(); } fn fourth() {} }",
        1,
    );
}

#[test]
fn given_unreachable_cycles_and_indirect_calls_when_checked_then_terminates_and_ignores_unknown_calls()
 {
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "fn first() { second(); } fn second() { third(); second(); } fn third() { second(); } fn other() { (factory())(); } impl Subject { const LIMIT: usize = 1; fn read(&self) { other.read(); } }",
        1,
    );
}

#[test]
fn given_recursive_scopes_and_constructors_when_checked_then_keeps_exempt_edges() {
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "fn first() { second(); } fn second() { third(); } fn third() { first(); } impl Subject { fn new() -> Self { Self::helper(); Self } fn helper() {} fn recurse(&self) { self.recurse(); } } impl Other for Subject { fn call() { Subject::other(); } }",
        0,
    );
}

#[test]
fn given_calls_in_macros_closures_and_async_when_checked_then_reports_later_callees() {
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "mod inline { fn first() { let _ = || second(); let _ = async { second(); }; check!(second()); invalid!(=>); block!({ second(); }); } fn second() {} } impl Subject { fn first(&self) { self.second(); } fn second(&self) {} fn third() { Subject::fourth(); } fn fourth() {} }",
        3,
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("CalleeBeforeCaller", "fail-syntax");
    assert!(!CalleeBeforeCaller.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("CalleeBeforeCaller", "pass-syntax");
    assert!(CalleeBeforeCaller.check(&context).is_empty());
}
