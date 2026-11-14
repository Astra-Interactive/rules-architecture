use crate::runtime::rule::BlockOnOutsideBoundary;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_fixture_messages;

struct RuleFixture<'rule> {
    id: &'rule str,
    rule: &'rule dyn crate::check::api::Rule,
}

#[test]
fn given_runtime_construction_and_blocking_outside_main_when_checked_then_each_names_the_boundary()
{
    let messages = read_fixture_messages(&BlockOnOutsideBoundary, "BlockOnOutsideBoundary");
    let lines: Vec<u32> = messages.iter().map(|reported| reported.line).collect();
    assert_eq!(vec![2, 5, 6, 7, 8], lines);
    for reported in &messages {
        if reported.line != 2 {
            assert_eq!(
                "Build and block a runtime only in entry-point root main; technology boundaries may block_on (di-manual.rs.md)",
                reported.message,
                "line {}",
                reported.line
            );
        }
    }
}

fn build_boundary_fixtures() -> Vec<RuleFixture<'static>> {
    use crate::runtime::rule::{
        BlockOnInTest, BlockingCallInAsync, DurationOverloadCall, ThreadPrimitive, WatchTimeout,
    };
    vec![
        RuleFixture {
            id: "BlockOnOutsideBoundary",
            rule: &BlockOnOutsideBoundary,
        },
        RuleFixture {
            id: "BlockOnInTest",
            rule: &BlockOnInTest,
        },
        RuleFixture {
            id: "BlockingCallInAsync",
            rule: &BlockingCallInAsync,
        },
        RuleFixture {
            id: "ThreadPrimitive",
            rule: &ThreadPrimitive,
        },
        RuleFixture {
            id: "WatchTimeout",
            rule: &WatchTimeout,
        },
        RuleFixture {
            id: "DurationOverloadCall",
            rule: &DurationOverloadCall,
        },
    ]
}

fn build_lock_fixtures() -> Vec<RuleFixture<'static>> {
    use crate::runtime::rule::{
        BlockingLockInSync, BroadcastLag, CriticalSectionWrite, ImpureUpdateClosure,
        JoinSetBehindLock, LockChoice, SelectCancelSafety,
    };
    vec![
        RuleFixture {
            id: "JoinSetBehindLock",
            rule: &JoinSetBehindLock,
        },
        RuleFixture {
            id: "CriticalSectionWrite",
            rule: &CriticalSectionWrite,
        },
        RuleFixture {
            id: "LockChoice",
            rule: &LockChoice,
        },
        RuleFixture {
            id: "BroadcastLag",
            rule: &BroadcastLag,
        },
        RuleFixture {
            id: "SelectCancelSafety",
            rule: &SelectCancelSafety,
        },
        RuleFixture {
            id: "ImpureUpdateClosure",
            rule: &ImpureUpdateClosure,
        },
        RuleFixture {
            id: "BlockingLockInSync",
            rule: &BlockingLockInSync,
        },
    ]
}

fn list_placeholder_messages(fixtures: Vec<RuleFixture<'_>>) -> Vec<String> {
    let mut offenders = Vec::new();
    for fixture in fixtures {
        let messages = read_fixture_messages(fixture.rule, fixture.id);
        assert!(
            !messages.is_empty(),
            "{} reports nothing on its fixture",
            fixture.id
        );
        for reported in messages {
            if reported.message.contains("Follow the") || reported.message.contains(fixture.id) {
                offenders.push(format!(
                    "{}:{}: {}",
                    fixture.id, reported.line, reported.message
                ));
            }
        }
    }
    offenders
}

#[test]
fn given_failing_fixture_of_every_boundary_runtime_rule_when_checked_then_no_message_is_a_placeholder()
 {
    let offenders = list_placeholder_messages(build_boundary_fixtures());
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}

#[test]
fn given_failing_fixture_of_every_lock_and_channel_rule_when_checked_then_no_message_is_a_placeholder()
 {
    let offenders = list_placeholder_messages(build_lock_fixtures());
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&BlockOnOutsideBoundary, "BlockOnOutsideBoundary", "bad");
    assert_code_fixture(&BlockOnOutsideBoundary, "BlockOnOutsideBoundary", "good");
}
use crate::check::api::Rule;
use crate::config::model::Role;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_entry_root_main_when_checked_then_accepts_runtime_construction_and_blocking() {
    let mut context = read_code_context(
        "use tokio::runtime::Builder;\nfn main() { let runtime = Builder::new_multi_thread().build(); runtime.block_on(work()); }\nfn other() { futures::executor::block_on(work()); }",
    );
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    let findings = BlockOnOutsideBoundary.check(&context);
    assert_eq!(1, findings.len());
    assert_eq!(Some(3), findings[0].finding.location.line);
    assert_eq!("src/lib.rs", findings[0].finding.location.path);
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "use tokio::runtime::{Runtime as AsyncRuntime, Builder as AsyncBuilder};\n#[tokio::main]\nasync fn boot() {}\nfn run() {\n    let runtime = AsyncRuntime::new();\n    let runtime = AsyncBuilder::new_multi_thread().build().unwrap();\n    runtime.block_on(work());\n    futures::executor::block_on(work());\n}",
        &[2, 5, 6, 7, 8],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "mod file {\n    fn run(handle: tokio::runtime::Handle) { handle.block_on(work()); }\n}\nstruct Builder;\nimpl Builder { fn new_current_thread() {} }\nfn run() { Builder::new_current_thread(); }",
        &[],
    );
}

#[test]
fn given_blocking_test_when_checked_then_is_also_outside_the_runtime_boundary() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "#[cfg(test)] mod test { fn run(runtime: tokio::runtime::Runtime) { runtime.block_on(work()); } }",
        &[1],
    );
}

#[test]
fn given_result_checked_runtime_constructors_when_checked_then_reports_constructor_and_blocking() {
    for (source, lines) in [
        (
            "fn run() {\n    let runtime = match tokio::runtime::Runtime::new() {\n        Ok(runtime) => runtime,\n        Err(_) => return,\n    };\n    runtime.block_on(async {});\n}",
            &[2, 6][..],
        ),
        (
            "use tokio::runtime::Builder;\nfn run() {\n    let runtime = match Builder::new_current_thread().enable_all().build() {\n        Ok(runtime) => runtime,\n        Err(_) => return,\n    };\n    runtime.block_on(async {});\n}",
            &[3, 7][..],
        ),
        (
            "use tokio::runtime::Builder;\nfn run() -> Result<(), ()> {\n    let runtime = Builder::new_current_thread().enable_all().build().map_err(|_| ())?;\n    runtime.block_on(async {});\n    Ok(())\n}",
            &[3, 4][..],
        ),
    ] {
        assert_code_findings(
            &BlockOnOutsideBoundary,
            "BlockOnOutsideBoundary",
            source,
            lines,
        );
    }
}

#[test]
fn given_result_type_alias_when_checked_then_classifies_the_matched_value_type() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "type Outcome<E> = Result<tokio::runtime::Runtime, E>;\nfn run(input: Outcome<()>) {\n    let runtime = match input {\n        Ok(runtime) => runtime,\n        Err(_) => return,\n    };\n    runtime.block_on(async {});\n}",
        &[7],
    );
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "struct Custom;\nimpl Custom { fn block_on(&self) {} }\ntype Outcome<E> = Result<Custom, E>;\nfn run(input: Outcome<tokio::runtime::Runtime>) {\n    let custom = match input {\n        Ok(value) => value,\n        Err(_) => return,\n    };\n    custom.block_on();\n}",
        &[],
    );
}

#[test]
fn given_conditional_tokio_main_attribute_when_checked_then_reports_the_attribute() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "#[cfg_attr(not(test), tokio::main)]\nasync fn main() {}",
        &[1],
    );
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "#[cfg_attr(not(test), inline)]\nasync fn main() {}",
        &[],
    );
}

#[test]
fn given_dependency_block_on_function_or_local_set_method_when_checked_then_reports_the_call() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "mod usecase {\n    pub(crate) fn load_blocking(client: &Client) -> Offer {\n        vendor::block_on(client.load_offer())\n    }\n    pub(crate) fn run_local(set: tokio::task::LocalSet) {\n        set.block_on(work());\n    }\n}\nstruct Queue;\nimpl Queue { fn block_on(&self, _: u32) {} }\nfn drain(queue: Queue) { queue.block_on(1); }",
        &[3, 6],
    );
}

#[test]
fn given_cross_module_result_aliases_when_checked_then_uses_the_alias_owner_type() {
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "mod api {\n    pub struct Custom;\n    impl Custom { pub fn block_on(&self) {} }\n    pub type Outcome<E> = Result<Custom, E>;\n}\nfn run(input: api::Outcome<tokio::runtime::Runtime>) {\n    let custom = match input {\n        Ok(value) => value,\n        Err(_) => return,\n    };\n    custom.block_on();\n}",
        &[],
    );
    assert_code_findings(
        &BlockOnOutsideBoundary,
        "BlockOnOutsideBoundary",
        "mod api {\n    pub type Outcome<E> = Result<tokio::runtime::Runtime, E>;\n}\nfn run(input: api::Outcome<()>) {\n    let runtime = match input {\n        Ok(value) => value,\n        Err(_) => return,\n    };\n    runtime.block_on(async {});\n}",
        &[9],
    );
}

#[test]
fn given_runtime_built_in_main_of_a_binary_entry_point_with_a_library_when_checked_then_main_is_the_boundary()
 {
    let context =
        crate::test::check::usecase::read_case_context("CrateTargets", "pass-binary-entry-point");
    assert!(crate::check::api::Rule::check(&BlockOnOutsideBoundary, &context).is_empty());
}
