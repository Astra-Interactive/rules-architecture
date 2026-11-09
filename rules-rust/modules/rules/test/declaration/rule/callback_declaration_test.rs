use crate::check::api::Rule;
use crate::declaration::rule::CallbackDeclaration;
use crate::source::model::SourceTree;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_macro_expansion_test_target_when_checked_then_does_not_report_callback_declarations() {
    let mut context = read_code_context("trait TimerListener {} fn run(on_event: fn()) {}");
    for module in &mut context.indexes[0].modules {
        module.tree = SourceTree::TestsIt;
    }
    assert!(CallbackDeclaration.check(&context).is_empty());
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&CallbackDeclaration, "CallbackDeclaration", "bad");
    assert_code_fixture(&CallbackDeclaration, "CallbackDeclaration", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &CallbackDeclaration,
        "CallbackDeclaration",
        "trait TimerListener { fn tick(&self); }\ntrait CompletionCallback {}\nfn load(on_result: impl FnOnce(String)) {}\nfn load_generic<F>(on_result: F) where F: FnMut(String) {}\nstruct Gateway<F: Fn()> { on_event: Option<Box<dyn Fn()>>, on_done: F }",
        &[1, 2, 3, 4, 5, 5],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &CallbackDeclaration,
        "CallbackDeclaration",
        "fn map(transform: impl Fn(String)) {}\nimpl External for Gateway { fn run(&self, on_result: impl Fn()) {} }\nmod test { trait TimerListener {} fn run(on_event: fn()) {} }\nmod fixture { trait CompletionCallback {} struct Sample { on_event: fn() } }",
        &[],
    );
}

#[test]
fn given_collections_and_enum_payloads_when_checked_then_does_not_expand_callback_scope() {
    assert_code_findings(
        &CallbackDeclaration,
        "CallbackDeclaration",
        "enum Event { Dynamic { on_event: fn() } }\nstruct Callbacks { on_events: Vec<Box<dyn Fn()>> }",
        &[],
    );
    assert_code_findings(
        &CallbackDeclaration,
        "CallbackDeclaration",
        "impl External for Gateway { fn run(&self, on_result: fn()) { fn nested(on_event: fn()) {} } }",
        &[1],
    );
}
