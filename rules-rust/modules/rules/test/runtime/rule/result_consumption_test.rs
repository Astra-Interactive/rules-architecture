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

#[test]
fn given_tdd_result_consumption_fallible_loop_initializer_when_linted_then_reports_let() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn first(value: u32) -> Result<u32, ()> { value.checked_add(1).ok_or(()) }\nfn second(value: u32) -> Result<u32, ()> { value.checked_add(2).ok_or(()) }\nfn compute(values: &mut impl Iterator<Item = u8>, left: u32, right: u32) -> Result<u32, ()> {\n    let value = loop { match values.next() { Some(0) => continue, Some(_) => break first(left)? + second(right)?, None => return Ok(0) } };\n    Ok(value)\n}",
        &[4],
    );
}

#[test]
fn given_result_consumption_sender_from_take_when_linted_then_reports_only_the_real_discard() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "struct Gate { slot: parking_lot::Mutex<Option<tokio::sync::oneshot::Sender<bool>>> }\nimpl Gate {\n    fn complete(&self, value: bool) {\n        let Some(sender) = self.slot.lock().take() else { return; };\n        #[expect(clippy::let_underscore_must_use, reason = \"the receiver is gone only when the caller was cancelled\")] let _ = sender.send(value);\n    }\n    fn load(&self) {\n        #[expect(clippy::let_underscore_must_use, reason = \"too broad\")] let _ = std::fs::read(\"file\");\n    }\n}",
        &[8],
    );
}

#[test]
fn given_result_consumption_composed_tail_expression_when_linted_then_reports_the_expression() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn parse(_: &str) -> Result<u8, ()> { Ok(1) }\nfn validate(value: u8) -> Result<u8, ()> { Ok(value) }\nfn run(raw: &str) -> Result<u8, ()> {\n    Ok(validate(parse(raw)?)?)\n}\nfn run_separately(raw: &str) -> Result<u8, ()> {\n    let parsed = parse(raw)?;\n    Ok(validate(parsed)?)\n}",
        &[4],
    );
}

#[test]
fn given_result_consumption_loop_tail_with_separate_fallible_statements_when_linted_then_reports_only_the_composed_let()
 {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn read() -> Result<Option<u8>, ()> { Ok(Some(1)) }\nfn send(_: u8) -> Result<(), ()> { Ok(()) }\nfn load() -> Result<u8, ()> { Ok(1) }\nfn retain(value: u8) -> Result<u8, ()> { Ok(value) }\nfn walk(ready: bool) -> Result<(), ()> {\n    loop {\n        let Some(kind) = read()? else {\n            continue;\n        };\n        match kind {\n            1 => {\n                if ready {\n                    send(1)?;\n                } else {\n                    return Ok(());\n                }\n            }\n            2 => {\n                send(2)?;\n            }\n            3 => {\n                send(3)?;\n                return Ok(());\n            }\n            _ => {}\n        }\n    }\n}\nfn compose() -> Result<u8, ()> {\n    let value = retain(load()?)?;\n    Ok(value)\n}",
        &[30],
    );
}

#[test]
fn given_result_consumption_match_tail_when_linted_then_reports_the_composing_arm() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn send(_: u8) -> Result<(), ()> { Ok(()) }\nfn load() -> Result<u8, ()> { Ok(1) }\nfn retain(value: u8) -> Result<u8, ()> { Ok(value) }\nfn pick(ready: bool) -> Result<u8, ()> {\n    match ready {\n        true => {\n            send(1)?;\n            send(2)?;\n            Ok(1)\n        }\n        false => Ok(retain(load()?)?),\n    }\n}",
        &[11],
    );
}

#[test]
fn given_result_consumption_underscore_named_send_result_when_linted_then_reports_the_let() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn deliver(sender: tokio::sync::oneshot::Sender<bool>, outcome: bool) {\n    let _outcome = sender.send(outcome);\n}\nstruct CustomSender;\nimpl CustomSender { fn send(&self, _: bool) -> Result<(), ()> { Ok(()) } }\nfn custom(sender: CustomSender) {\n    let _outcome = sender.send(true);\n}",
        &[2, 7],
    );
}

#[test]
fn given_result_consumption_underscore_assignment_of_workspace_result_when_linted_then_reports_the_statement()
 {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn call() -> Result<u8, ()> { Ok(1) }\nfn run() -> Result<(), ()> {\n    _ = call();\n    let saved = call()?;\n    Ok(())\n}",
        &[3],
    );
}

#[test]
fn given_result_consumption_sender_through_nested_field_when_linted_then_reports_only_the_real_discard()
 {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "struct Channels { events: tokio::sync::mpsc::Sender<u32> }\nstruct Holder { channels: Channels }\nimpl Holder {\n    fn on_paid(&self) {\n        #[expect(clippy::let_underscore_must_use, reason = \"a full buffer drops the event\")] let _ = self.channels.events.try_send(1);\n    }\n    fn load(&self) {\n        #[expect(clippy::let_underscore_must_use, reason = \"too broad\")] let _ = std::fs::read(\"file\");\n    }\n}",
        &[8],
    );
}

#[test]
fn given_workspace_result_dropped_through_ok_when_checked_then_reports_statement_and_let() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn save_order() -> Result<(), ()> { Ok(()) }\nfn persist() {\n    save_order().ok();\n    let _ = save_order().ok();\n    let saved = save_order().ok();\n    drop(saved);\n}",
        &[3, 4],
    );
}

#[test]
fn given_underscore_named_binding_of_workspace_result_when_checked_then_reports_unread_binding() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn call() -> Result<u8, ()> { Ok(1) }\nstruct Guard;\nimpl Guard { fn new() -> Self { Self } }\nfn run() -> Result<(), ()> {\n    let _ignored = call();\n    let _registration = Guard::new();\n    let _saved = call()?;\n    let _kept = call();\n    drop(_kept);\n    Ok(())\n}",
        &[5],
    );
}

#[test]
fn given_underscore_binding_read_through_format_argument_when_checked_then_reports_only_unread() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn load_config() -> Result<u8, ()> { Ok(1) }\nfn inline() {\n    let _error = load_config();\n    tracing::warn!(\"config result: {_error:?}\");\n}\nfn positional() {\n    let _error = load_config();\n    tracing::warn!(\"config result: {:?}\", _error);\n}\nfn never_read() {\n    let _ignored = load_config();\n    tracing::warn!(\"config loaded\");\n}",
        &[11],
    );
}

#[test]
fn given_ok_of_workspace_result_assigned_to_underscore_when_checked_then_reports_the_statement() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn save_order() -> Result<(), ()> { Ok(()) }\nfn persist() {\n    _ = save_order().ok();\n    let kept;\n    kept = save_order().ok();\n    drop(kept);\n}",
        &[3],
    );
}

#[test]
fn given_underscore_binding_shadowed_before_any_read_when_checked_then_reports_the_first_binding() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn call() -> Result<u8, ()> { Ok(1) }\nfn run() {\n    let _outcome = call();\n    let _outcome = 1_u8;\n    drop(_outcome);\n}\nfn chained() {\n    let _outcome = call();\n    let _outcome = _outcome.is_ok();\n    drop(_outcome);\n}",
        &[3],
    );
}

#[test]
fn given_workspace_result_of_a_method_associated_or_async_function_when_dropped_then_reports_each()
{
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "struct InvoiceStore;\nimpl InvoiceStore {\n    fn save(&self) -> Result<(), ()> { Ok(()) }\n    fn open() -> Result<Self, ()> { Ok(Self) }\n    fn count(&self) -> u8 { 1 }\n}\nasync fn persist_later() -> Result<(), ()> { Ok(()) }\nasync fn persist(store: &InvoiceStore) {\n    _ = store.save();\n    let _saved = store.save();\n    _ = persist_later().await;\n    _ = InvoiceStore::open();\n    store.save().ok();\n    _ = store.count();\n    let _pending = persist_later();\n}",
        &[9, 10, 11, 12, 13],
    );
}

#[test]
fn given_proven_foreign_result_when_dropped_by_underscore_binding_or_ok_then_reports_each() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "async fn persist(path: &std::path::Path, handle: tokio::task::JoinHandle<u8>) {\n    _ = std::fs::write(path, b\"x\");\n    _ = tokio::fs::write(path, b\"x\").await;\n    _ = handle.await;\n    let _removed = std::fs::remove_file(path);\n    std::fs::remove_file(path).ok();\n    _ = std::fs::read(path).ok();\n    let _ = tokio::fs::read(path).await.ok();\n    let _pending = tokio::fs::write(path, b\"x\");\n    let _kept = std::fs::read(path);\n    drop(_kept);\n    _ = path.exists();\n}",
        &[2, 3, 4, 5, 6, 7, 8],
    );
}

#[test]
fn given_awaited_method_returning_join_handle_when_dropped_then_reports_each_and_not_the_unawaited()
{
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "use tokio::task::JoinHandle;\nstruct Worker;\nimpl Worker {\n    fn start(&self) -> JoinHandle<u8> { loop {} }\n}\nasync fn run(worker: &Worker) {\n    _ = worker.start().await;\n    let _done = worker.start().await;\n    worker.start().await.ok();\n    let _pending = worker.start();\n}",
        &[7, 8, 9],
    );
}

#[test]
fn given_two_fallible_steps_in_one_statement_when_checked_then_reports_the_statement() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn parse(_: &str) -> Result<u8, ()> { Ok(1) }\nfn save(_: u8) -> Result<(), ()> { Ok(()) }\nfn run(raw: &str) -> Result<(), ()> {\n    save(parse(raw)?)?;\n    let parsed = parse(raw)?;\n    save(parsed)?;\n    Ok(())\n}\nfn stop(raw: &str) -> Result<(), ()> {\n    if raw.is_empty() {\n        return Ok(save(parse(raw)?)?);\n    }\n    Ok(())\n}",
        &[4, 11],
    );
}

#[test]
fn given_tdd_result_consumption_match_guard_and_arm_when_linted_then_reports_let() {
    assert_code_findings(
        &ResultConsumption,
        "ResultConsumption",
        "fn condition(raw: &str) -> Result<bool, ()> { raw.parse::<u8>().map(|value| value > 0).map_err(|_| ()) }\nfn step(raw: &str) -> Result<usize, ()> { raw.parse::<usize>().map_err(|_| ()) }\nfn compute(number: u8, raw: &str) -> Result<usize, ()> {\n    let value = match number { 0..=2 if condition(raw)? => 0, 1 => step(raw)?, _ => 0 };\n    Ok(value)\n}",
        &[4],
    );
}
