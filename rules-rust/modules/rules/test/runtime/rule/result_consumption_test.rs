use crate::runtime::rule::ResultConsumption;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

const FALLIBLE_FUNCTIONS: &str = "fn load() -> Result<u8, ()> { Ok(1) } fn parse(_: &str) -> Result<u8, ()> { Ok(1) } fn retain(value: u8) -> Result<u8, ()> { Ok(value) } fn combine(left: u8, right: u8) -> u8 { left + right } fn inspect(_: ()) -> Result<(), ()> { Ok(()) } fn annotate(_: ()) -> Result<(), ()> { Ok(()) }";

#[test]
fn given_option_steps_in_one_let_when_checked_then_reports_the_let() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn parse(raw: &str) -> Option<u64> {\n    let value = raw.split_whitespace().nth(1)?.parse::<u64>().ok()?;\n    Some(value)\n}",
        &[2],
    );
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn parse(raw: &str) -> Option<u64> {\n    let token = raw.split_whitespace().nth(1)?;\n    let value = token.parse::<u64>().ok()?;\n    Some(value)\n}",
        &[],
    );
}

#[test]
fn given_composed_fallible_initializer_when_checked_then_reports_the_let() {
    for initializer in [
        "retain(parse(raw)?)?",
        "combine(load()?, parse(raw)?)",
        "{ retain((parse(raw)?))? }",
        "if ready { retain(parse(raw)?)? } else { 0 }",
        "match ready { true => retain(parse(raw)?)?, false => 0 }",
        "match ready { _ if load()? > 0 => parse(raw)?, _ => 0 }",
        "match ready { _ if load()? > 0 => 0, _ if parse(raw)? > 0 => 0, _ => 0 }",
        "match ready { _ if (load()? > 0) && false => 0, _ => parse(raw)? }",
        "'done: { break 'done retain(load()?)? }",
        "retain('done: { if ready { load()?; break 'done 0; } parse(raw)? })?",
        "retain('done: { if ready { break 'done 0; } load()?; let marker = { break 'done 0 }; 0 })?",
        "match ready { _ if load()? > 0 => 0, _ => parse(raw)? }",
        "{ ready && { return Ok(()); }; retain(parse(raw)?)? }",
    ] {
        assert_code_findings(
            &ResultConsumption,
            "ResultConsumption",
            &format!(
                "fn run(raw: &str, ready: bool) -> Result<(), ()> {{\nlet value = {initializer};\nOk(()) }}\n{FALLIBLE_FUNCTIONS}"
            ),
            &[2],
        );
    }
}

#[test]
fn given_separate_fallible_scopes_when_checked_then_does_not_combine_their_steps() {
    for body in [
        "let parsed = parse(raw)?; let retained = retain(parsed)?;",
        "let value = if ready { load()? } else { parse(raw)? };",
        "let value = match ready { true => load()?, false => parse(raw)? };",
        "let value = match ready { true if load()? > 0 => 0, false => parse(raw)?, _ => 0 };",
        "let value = { let parsed = parse(raw)?; retain(parsed)? };",
        "let callback = || { load()?; parse(raw)?; Ok(()) };",
        "let future = async { load()?; parse(raw)?; Ok(()) };",
        "let value = if false { retain(parse(raw)?)? } else { 0 };",
        "let value = { if ready { load()?; return Ok(()); } parse(raw)? };",
        "let value = { match ready { true => { load()?; return Ok(()); }, false => {} } parse(raw)? };",
        "let value = 'done: { if ready { load()?; break 'done 0; } parse(raw)? };",
        "let value = retain('done: { if ready { break 'done 0; } load()?; let marker = 0; return Ok(()); })?;",
        "let value = 'outer: { 'inner: { if ready { load()?; break 'outer 0; } parse(raw)? } };",
        "let value = match ready { true if load()? > 0 => 0, false => parse(raw)?, _ => 0 };",
        "let value = match ready { _ => load()?, _ => retain(parse(raw)?)? };",
        "let value = 'done: { let nested = 'done: { load()?; break 'done 0 }; parse(raw)? };",
        "let value = { if ready { return Ok(()); } else { return Ok(()); } retain(parse(raw)?)? };",
        "let value = (false && (load()? > 0)) || (parse(raw)? > 0);",
        "let value = (true || (load()? > 0)) && (parse(raw)? > 0);",
        "let value = (!(true) && (load()? > 0)) || (parse(raw)? > 0);",
        "let value = { let stopped = { return Ok(()); }; retain(parse(raw)?)? };",
        "let value = match ready { _ if false => retain(load()?)?, _ => 0 };",
        "let value = load().map_err(|error| { inspect(error)?; annotate(error)?; Ok(()) })?;",
    ] {
        assert_code_findings(
            &ResultConsumption,
            "ResultConsumption",
            &format!(
                "fn run(raw: &str, ready: bool) -> Result<(), ()> {{ {body} Ok(()) }}\n{FALLIBLE_FUNCTIONS}"
            ),
            &[],
        );
    }
}

#[test]
fn given_nested_fallible_let_when_checked_then_checks_its_own_execution_scope() {
    for wrapper in [
        "let callback = || {",
        "let future = async {",
        "let value = {",
        "let Some(value) = input else {",
    ] {
        assert_code_findings(
            &ResultConsumption,
            "ResultConsumption",
            &format!(
                "fn run(input: Option<u8>) -> Result<(), ()> {{ {wrapper}\nlet value = retain(load()?)?;\nreturn Err(()); }}; Ok(()) }}\n{FALLIBLE_FUNCTIONS}"
            ),
            &[2],
        );
    }
}

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
