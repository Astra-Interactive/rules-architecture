use crate::runtime::rule::ResultConsumption;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_sender_discard_expectation_when_checked_then_requires_one_of_four_resolved_apis() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "use tokio::sync::oneshot::channel as open; fn run() { let (sender, receiver) = open::<u32>();\n #[expect(clippy::let_underscore_must_use, reason = \"receiver is gone\")] let _ = sender.send(1);\n}",
        &[],
    );
    for typed in [
        "tokio::sync::oneshot::Sender<u32>",
        "tokio::sync::mpsc::UnboundedSender<u32>",
        "tokio::sync::broadcast::Sender<u32>",
    ] {
        assert_code_findings(
            &ResultConsumption,
            "ResultConsumption",
            &format!(
                "fn run(sender: {typed}) {{\n #[expect(clippy::let_underscore_must_use, reason = \"receiver is gone\")] let _ = sender.send(1);\n}}"
            ),
            &[],
        );
    }
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn run(sender: tokio::sync::mpsc::Sender<u32>) {\n #[expect(clippy::let_underscore_must_use, reason = \"full buffer\")] let _ = sender.try_send(1);\n}",
        &[],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "struct CustomSender; impl CustomSender { fn new() -> Self { Self } fn send(&self, _: u32) -> Result<(), ()> { Ok(()) } } fn run() { let sender = CustomSender::new();\n #[expect(clippy::let_underscore_must_use, reason = \"receiver is gone\")] let _ = sender.send(1);\n}",
        &[2],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn run(sender: tokio::sync::mpsc::Sender<u32>) {\n #[expect(clippy::let_underscore_must_use, reason = \"too broad\")] let _ = sender.send(1);\n}",
        &[2],
    );
}

#[test]
fn given_typed_runtime_cases_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "#[expect(clippy::let_underscore_must_use, reason = \"too wide\")]\nfn run() {}\n",
        &[1],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn run() {\n    #[expect(clippy::let_underscore_must_use, reason = \"too broad\")]\n    let _ = std::fs::read(\"file\");\n}\n",
        &[2],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn run(sender: tokio::sync::oneshot::Sender<u32>) {\n    #[expect(clippy::let_underscore_must_use, reason = \"receiver is gone\")]\n    let _ = sender.send(1);\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ResultConsumption, "ResultConsumption", "bad");
    assert_code_fixture(&ResultConsumption, "ResultConsumption", "good");
}
