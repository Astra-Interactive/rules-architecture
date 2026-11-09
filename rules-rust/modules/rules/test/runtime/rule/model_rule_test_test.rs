use crate::runtime::rule::ModelRuleTest;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_pure_model_method_in_assertion_when_using_tokio_then_requires_synchronous_test() {
    let model = "mod model { pub struct Money { pub cents: i64 } } mod util { impl crate::model::Money { pub fn is_valid(&self) -> bool { self.cents >= 0 } } }";
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        &format!(
            "{model}\n#[cfg(test)] mod test {{ mod model {{\n #[tokio::test]\n async fn verify() {{ let money = crate::model::Money {{ cents: 1 }}; assert!(money.is_valid()); }}\n}} }}"
        ),
        &[3],
    );
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        &format!(
            "{model}\n#[cfg(test)] mod test {{\n #[tokio::test]\n async fn verify() {{ let money = crate::model::Money {{ cents: 1 }}; assert!(money.is_valid()); tokio::fs::read(\"file\").await; }}\n}}"
        ),
        &[],
    );
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "mod service { pub struct Store; impl Store { pub fn is_valid(&self) -> bool { std::fs::read(\"file\").is_ok() } } }\n#[cfg(test)] mod test { #[tokio::test] async fn verify() { let store = crate::service::Store; assert!(store.is_valid()); } }",
        &[],
    );
}

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "mod model { pub struct Money; } mod util { pub fn is_valid(_: &crate::model::Money) -> bool { true } } #[cfg(test)] mod test { mod util {\n    #[tokio::test]\n    async fn test() { let money = crate::model::Money; let valid = crate::util::is_valid(&money); assert!(valid); }\n} }\n",
        &[2],
    );
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "#[cfg(test)] mod test { mod util {\n    #[test]\n    fn test() {}\n} }\n",
        &[],
    );
}

#[test]
fn given_async_io_test_in_util_when_checked_then_does_not_classify_it_as_model_only() {
    assert_code_findings(
        &ModelRuleTest,
        "ModelRuleTest",
        "#[cfg(test)] mod test { mod util {\n #[tokio::test] async fn verify_io() { let bytes = tokio::fs::read(\"file\").await; assert!(bytes.is_ok()); }\n} }",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ModelRuleTest, "ModelRuleTest", "bad");
    assert_code_fixture(&ModelRuleTest, "ModelRuleTest", "good");
}
