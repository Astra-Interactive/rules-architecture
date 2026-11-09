use crate::runtime::rule::BlockOnOutsideBoundary;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

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
