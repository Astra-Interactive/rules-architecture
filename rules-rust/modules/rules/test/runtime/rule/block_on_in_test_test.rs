use crate::runtime::rule::BlockOnInTest;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&BlockOnInTest, "BlockOnInTest", "bad");
    assert_code_fixture(&BlockOnInTest, "BlockOnInTest", "good");
}

#[test]
fn given_runtime_type_alias_when_checked_then_resolves_constructor_to_tokio() {
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "#[cfg(test)] mod test { type Runner = tokio::runtime::Runtime; fn run() { let runtime = Runner::new(); } }",
        &[1],
    );
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "#[cfg(test)]\nmod test {\n    use tokio::runtime::{Runtime as AsyncRuntime, Builder as AsyncBuilder};\n    fn run() {\n        let runtime = AsyncRuntime::new();\n        let runtime = AsyncBuilder::new_current_thread().build().unwrap();\n        runtime.block_on(work());\n        futures::executor::block_on(work());\n    }\n}",
        &[5, 6, 7, 8],
    );
}

#[test]
fn given_unrelated_or_shadowed_block_on_receiver_when_checked_then_allows_project_method() {
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "struct Queue; impl Queue { fn block_on(&self, _: u32) {} }\n#[cfg(test)] mod test { fn verify(runtime: tokio::runtime::Runtime) {\n let queue = crate::Queue; queue.block_on(1);\n { let runtime = queue; runtime.block_on(2); }\n runtime.block_on(async {});\n} }",
        &[5],
    );
}

#[test]
fn given_handle_result_and_same_named_project_methods_when_checked_then_reports_only_tokio_blocking()
 {
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "#[cfg(test)]\nmod test {\n    fn run() -> Result<(), ()> {\n        let handle = tokio::runtime::Handle::try_current().map_err(|_| ())?;\n        handle.block_on(async {});\n        Ok(())\n    }\n}",
        &[5],
    );
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "#[cfg(test)]\nmod test {\n    struct Value;\n    impl Value { fn block_on(&self, _: u32) {} }\n    fn run(value: Value) { value.block_on(1); }\n}",
        &[],
    );
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "#[cfg(test)]\nmod test {\n    struct Custom;\n    impl Custom { fn block_on(&self, _: u32) {} }\n    fn run() {\n        let runtime = match tokio::runtime::Runtime::new() {\n            Ok(runtime) => { let runtime = Custom; runtime },\n            Err(_) => Custom,\n        };\n        runtime.block_on(1);\n    }\n}",
        &[6],
    );
}

#[test]
fn given_dependency_block_on_function_in_test_when_checked_then_reports_whatever_the_crate() {
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "#[cfg(test)]\nmod test {\n    fn run() {\n        vendor::block_on(work());\n        vendor::future::block_on(work());\n        let mut task = vendor::task::spawn(work());\n    }\n}",
        &[4, 5],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &BlockOnInTest,
        "BlockOnInTest",
        "fn run() { futures::executor::block_on(work()); }\n#[cfg(test)]\nmod test {\n    #[tokio::test]\n    async fn run() { work().await; }\n}",
        &[],
    );
}
