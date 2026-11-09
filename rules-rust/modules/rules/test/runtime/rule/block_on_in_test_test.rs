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
        "#[cfg(test)]\nmod test {\n    use tokio::runtime::{Runtime as AsyncRuntime, Builder as AsyncBuilder};\n    fn run() {\n        let runtime = AsyncRuntime::new();\n        let runtime = AsyncBuilder::new_current_thread();\n        runtime.block_on(work());\n        futures::executor::block_on(work());\n    }\n}",
        &[5, 6, 7, 8],
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
