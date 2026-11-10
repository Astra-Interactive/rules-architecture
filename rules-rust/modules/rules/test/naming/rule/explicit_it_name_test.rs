use crate::check::api::Rule;
use crate::naming::rule::ExplicitItName;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_item_import_and_generic_names_when_checked_then_reports_project_owned_it_names() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExplicitItName,
        "ExplicitItName",
        "fn it() {} mod it {} struct it; enum Choice { it } trait it {} type it = u64; const it: u64 = 1; static it: u64 = 1; use external::Value as it; use external::it; fn work<it, const it: usize>() {} macro_rules! it { () => {} }",
        13,
    );
    crate::test::syntax::util::assert_syntax_findings(
        &ExplicitItName,
        "ExplicitItName",
        "struct Value; impl External for Value { fn it(&self) {} const it: usize = 1; type it = usize; } use external::it as descriptive; fn work(_it: usize) {}",
        0,
    );
}

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
fn given_function_pointer_argument_names_when_checked_then_reports_it() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExplicitItName,
        "ExplicitItName",
        "type Callback = fn(it: u32) -> u32; type Named = fn(value: u32) -> u32; fn apply(callback: fn(it: u32) -> u32, value: u32) -> u32 { callback(value) }",
        2,
    );
}

#[test]
fn given_it_binding_inside_macro_pattern_with_guard_when_checked_then_reports_it() {
    crate::test::syntax::util::assert_syntax_findings(
        &ExplicitItName,
        "ExplicitItName",
        "fn inside(event: PointerEvent) -> bool { matches!(event, PointerEvent::Click(it) if it.x > 0) }\nfn inside_match(event: PointerEvent) -> bool { match event { PointerEvent::Click(it) if it.x > 0 => true, _ => false } }",
        2,
    );
}

#[test]
fn given_it_names_inside_tokio_select_when_checked_then_reports_each_position_but_not_futures() {
    crate::test::declaration::util::assert_code_findings(
        &ExplicitItName,
        "ExplicitItName",
        "pub(crate) async fn forward(mut receiver: tokio::sync::mpsc::Receiver<String>, mut other: tokio::sync::mpsc::Receiver<String>) -> Option<String> {\n    tokio::select! {\n        biased;\n        message = receiver.recv() => {\n            let it = message;\n            it\n        }\n        it = other.recv() => it,\n        Some(value) = other.recv(), if [1].iter().any(|it| *it > 0) => value,\n        _ = std::future::ready([1].iter().map(|it| *it)) => None,\n        else => { let it = 0; None }\n    }\n}\npub(crate) async fn other_select() {\n    futures::select! { message = first() => { let it = message; } }\n}",
        &[5, 8, 9, 10, 11],
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
