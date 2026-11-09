use crate::check::api::Rule;
use crate::ordering::rule::CalleeBeforeCaller;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_alternative_branches_when_checked_then_each_branch_sees_its_own_incoming_aliases() {
    for source in [
        "fn other() {} fn first(flag: bool) { let mut run = compute; if flag { run = other; } else { run(); } } fn compute() {}",
        "fn other() {} fn first(flag: bool) { let mut run = compute; match flag { true => { run = other; }, false => run() } } fn compute() {}",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &CalleeBeforeCaller,
            "CalleeBeforeCaller",
            source,
            1,
        );
    }
}

#[test]
fn given_pattern_scopes_when_checked_then_outer_callees_survive_only_outside_the_binding_scope() {
    for source in [
        "fn first() { for compute in [1] { let _ = compute; } compute(); } fn compute() {}",
        "fn first() { if let Some(compute) = Some(other) { compute(); } else { compute(); } } fn compute() {} fn other() {}",
        "fn first() { while let Some(compute) = None::<fn()> { compute(); break; } compute(); } fn compute() {}",
        "fn first(value: Option<fn()>) { match value { Some(compute) => compute(), None => compute() } } fn compute() {}",
        "fn first() { if let Some(compute) = { compute(); None::<fn()> } { compute(); } } fn compute() {}",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &CalleeBeforeCaller,
            "CalleeBeforeCaller",
            source,
            1,
        );
    }
    for source in [
        "fn other() {} fn first() { for compute in [other] { compute(); } } fn compute() {}",
        "fn other() {} fn first() { if let Some(compute) = Some(other) { compute(); } } fn compute() {}",
        "fn first(value: Option<fn()>) { match value { Some(compute) => compute(), None => {} } } fn compute() {}",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &CalleeBeforeCaller,
            "CalleeBeforeCaller",
            source,
            0,
        );
    }
}

#[test]
fn given_nested_writes_when_checked_then_invalidates_the_actual_binding_without_erasing_shadowed_aliases()
 {
    for source in [
        "fn other() {} fn first() { let mut run = compute; { run = other; } run(); } fn compute() {}",
        "fn other() {} fn first() { let mut run = compute; if true { run = other; } run(); } fn compute() {}",
        "fn other() {} fn first() { let mut run = compute; match true { true => { run = other; }, false => {} } run(); } fn compute() {}",
    ] {
        crate::test::syntax::util::assert_syntax_findings(
            &CalleeBeforeCaller,
            "CalleeBeforeCaller",
            source,
            0,
        );
    }
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "fn other() {} fn first() { let run = compute; { let mut run = other; run = other; } run(); } fn compute() {}",
        1,
    );
}

#[test]
fn given_local_function_aliases_when_checked_then_respects_shadowing_and_assignment() {
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "fn first() { let run = self::compute; let alias = run; alias(); } fn compute() {}\nmod shadow { fn first(compute: fn()) { compute(); } fn compute() {} }\nmod assigned { fn first() { let mut run = self::compute; run = external::compute; run(); } fn compute() {} }\nmod closure { fn first() { let compute = external::compute; compute(); } fn compute() {} }\nmod initializer { fn first() { let compute = compute(); } fn compute() -> u64 { 1 } }\nimpl Subject { fn first() { let call = Self::second; call(); } fn second() {} }",
        3,
    );
}

#[test]
fn given_same_module_qualified_calls_when_checked_then_distinguishes_other_modules() {
    crate::test::syntax::util::assert_syntax_findings(
        &CalleeBeforeCaller,
        "CalleeBeforeCaller",
        "fn first() { self::compute(); } fn compute() {}\nmod nested { fn first() { self::compute(); } fn compute() {} }\nmod other { fn first() { foreign::compute(); } fn compute() {} }",
        2,
    );
}

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
