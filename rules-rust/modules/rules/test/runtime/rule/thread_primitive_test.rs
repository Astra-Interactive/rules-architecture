use crate::check::api::Rule;
use crate::config::model::Role;
use crate::runtime::rule::ThreadPrimitive;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ThreadPrimitive,
        "ThreadPrimitive",
        "#[expect(clippy::disallowed_methods, reason = \"wrong boundary\")]\nfn run() { let thread = std::thread::spawn(|| {}); }\n",
        &[2],
    );
    assert_code_findings(
        &ThreadPrimitive,
        "ThreadPrimitive",
        "fn run() { let thread = std::thread::spawn(|| {}); }\n",
        &[],
    );
}

#[test]
fn given_named_thread_builder_spawn_with_expectation_when_checked_then_reports_the_spawn() {
    assert_code_findings(
        &ThreadPrimitive,
        "ThreadPrimitive",
        "#[expect(clippy::disallowed_methods, reason = \"runs until shutdown\")]\nfn start() -> std::io::Result<std::thread::JoinHandle<()>> {\n    std::thread::Builder::new().name(\"pump\".to_owned()).spawn(run_loop)\n}\nfn run_loop() {}\n",
        &[3],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ThreadPrimitive, "ThreadPrimitive", "bad");
    assert_code_fixture(&ThreadPrimitive, "ThreadPrimitive", "good");
}

#[test]
fn given_discarded_thread_handle_in_technology_module_when_checked_then_reports_each_spawn() {
    let expectation =
        "#[expect(clippy::disallowed_methods, reason = \"reads the device until shutdown\")]";
    let source = [
        "struct Device;",
        "impl Device { fn run(&self) {} }",
        expectation,
        "pub(crate) fn start(device: Device) {",
        "    std::thread::spawn(move || device.run());",
        "}",
        expectation,
        "pub(crate) fn start_ignored(device: Device) {",
        "    let _ = std::thread::spawn(move || device.run());",
        "}",
        expectation,
        "pub(crate) fn start_dropped(device: Device) {",
        "    drop(std::thread::spawn(move || device.run()));",
        "}",
        expectation,
        "pub(crate) fn start_named(device: Device) -> std::io::Result<()> {",
        "    std::thread::Builder::new().spawn(move || device.run())?;",
        "    Ok(())",
        "}",
        expectation,
        "pub(crate) fn start_kept(device: Device) -> std::thread::JoinHandle<()> {",
        "    let handle = std::thread::spawn(move || device.run());",
        "    handle",
        "}",
    ]
    .join("\n");
    let mut context = read_code_context(&source);
    context.project_config.crates[0].role = Some(Role::Technology);
    let mut lines = ThreadPrimitive
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(
        vec![Some(5), Some(9), Some(13), Some(17)],
        lines,
        "{source}"
    );
}

#[test]
fn given_thread_handle_returned_or_bound_in_technology_module_when_checked_then_reports_only_discard()
 {
    let expectation =
        "#[expect(clippy::disallowed_methods, reason = \"reads the device until shutdown\")]";
    let source = [
        "enum Mode { A, B }",
        "fn run() {}",
        "fn idle() {}",
        expectation,
        "pub(crate) fn start_returned() -> std::thread::JoinHandle<()> {",
        "    return std::thread::spawn(run);",
        "}",
        expectation,
        "pub(crate) fn start_named() -> std::io::Result<std::thread::JoinHandle<()>> {",
        "    return std::thread::Builder::new().name(\"worker\".to_owned()).spawn(run);",
        "}",
        expectation,
        "pub(crate) fn start_by_mode(mode: Mode) -> std::thread::JoinHandle<()> {",
        "    return match mode { Mode::A => std::thread::spawn(run), Mode::B => std::thread::spawn(idle) };",
        "}",
        expectation,
        "pub(crate) fn join_by_mode(mode: Mode) {",
        "    let handle = match mode { Mode::A => std::thread::spawn(run), Mode::B => std::thread::spawn(idle) };",
        "    let _joined = handle.join();",
        "}",
        expectation,
        "pub(crate) fn start_discarded() {",
        "    std::thread::spawn(run);",
        "}",
    ]
    .join("\n");
    let mut context = read_code_context(&source);
    context.project_config.crates[0].role = Some(Role::Technology);
    let mut lines = ThreadPrimitive
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(vec![Some(23)], lines, "{source}");
}

#[test]
fn given_builder_spawn_unwrapped_or_scoped_in_technology_module_when_checked_then_reports_builders()
{
    let expectation =
        "#[expect(clippy::disallowed_methods, reason = \"reads the device until shutdown\")]";
    let source = [
        "fn run() {}",
        expectation,
        "pub(crate) fn start_unwrapped() {",
        "    std::thread::Builder::new().spawn(run).unwrap();",
        "}",
        expectation,
        "pub(crate) fn start_expected() {",
        "    std::thread::Builder::new().spawn(run).expect(\"thread\");",
        "}",
        expectation,
        "pub(crate) fn start_scoped() {",
        "    std::thread::scope(|scope| { scope.spawn(run); });",
        "}",
    ]
    .join("\n");
    let mut context = read_code_context(&source);
    context.project_config.crates[0].role = Some(Role::Technology);
    let mut lines = ThreadPrimitive
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(vec![Some(4), Some(8)], lines, "{source}");
}

#[test]
fn given_thread_handle_assigned_to_underscore_in_technology_module_when_checked_then_reports_it() {
    let expectation =
        "#[expect(clippy::disallowed_methods, reason = \"reads the device until shutdown\")]";
    let source = [
        "struct Device;",
        "impl Device { fn run(&self) {} }",
        expectation,
        "pub(crate) fn start(device: Device) {",
        "    _ = std::thread::spawn(move || device.run());",
        "}",
        expectation,
        "pub(crate) fn start_named(device: Device) {",
        "    _ = std::thread::Builder::new().spawn(move || device.run());",
        "}",
        expectation,
        "pub(crate) fn start_kept(device: Device) -> std::thread::JoinHandle<()> {",
        "    let handle;",
        "    handle = std::thread::spawn(move || device.run());",
        "    handle",
        "}",
    ]
    .join("\n");
    let mut context = read_code_context(&source);
    context.project_config.crates[0].role = Some(Role::Technology);
    let mut lines = ThreadPrimitive
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(vec![Some(5), Some(9)], lines, "{source}");
}
