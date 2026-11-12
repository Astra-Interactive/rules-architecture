use crate::runtime::rule::ImpureUpdateClosure;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_fetch_update_closure_writing_a_field_of_self_when_checked_then_reports_each_write() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Meter { count: std::sync::atomic::AtomicUsize, last_seen: std::sync::atomic::AtomicUsize, queue: Vec<usize>, slot: Option<usize> }\nimpl Meter {\n    fn bump_store(&mut self) { let _ = self.count.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| { self.last_seen.store(current, std::sync::atomic::Ordering::Relaxed); Some(current + 1) }); }\n    fn bump_push(&mut self) { let _ = self.count.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| { self.queue.push(current); Some(current + 1) }); }\n    fn bump_take(&mut self) { let _ = self.count.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| { self.slot.take(); Some(current + 1) }); }\n    fn bump_read(&mut self) { let _ = self.count.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| { let seen = self.last_seen.load(std::sync::atomic::Ordering::Relaxed); Some(current + seen) }); }\n}",
        &[3, 4, 5],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Meter { state: tokio::sync::watch::Sender<u32>, ticks: std::sync::atomic::AtomicUsize }\nimpl Meter {\n    fn tick(&self) {\n        self.state.send_modify(|current| { self.ticks.fetch_add(1, std::sync::atomic::Ordering::Relaxed); *current += 1; });\n    }\n}",
        &[4],
    );
}

#[test]
fn given_update_closure_reading_another_channel_when_checked_then_reports_each_read() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn mirror(state: tokio::sync::watch::Sender<u32>, other: tokio::sync::watch::Receiver<u32>) {\n    state.send_modify(|current| { *current = *other.borrow(); });\n}\nfn drain(state: tokio::sync::watch::Sender<u32>, mut inbox: tokio::sync::mpsc::Receiver<u32>) {\n    state.send_modify(|current| { if let Ok(next) = inbox.try_recv() { *current = next; } });\n}\nfn block(state: tokio::sync::watch::Sender<u32>, mut inbox: tokio::sync::mpsc::UnboundedReceiver<u32>) {\n    state.send_if_modified(|current| { *current = inbox.blocking_recv().unwrap_or(0); true });\n}\nfn pure(state: tokio::sync::watch::Sender<u32>, step: u32) {\n    state.send_modify(|current| { *current += step; });\n}",
        &[2, 5, 8],
    );
}

#[test]
fn given_resolved_rwlock_or_captured_collection_when_updated_then_reports_effects_only_for_known_receivers()
 {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>, gate: parking_lot::RwLock<u32>) {\n state.send_modify(|current| { let guard = gate.read(); *current = *guard; });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(counter: std::sync::atomic::AtomicUsize, captured: &mut Vec<usize>) {\n counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured.push(current); Some(current) });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Buffer; impl Buffer { fn push(&self, _: usize) {} } fn run(counter: std::sync::atomic::AtomicUsize, captured: Buffer) {\n counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured.push(current); Some(current) });\n}",
        &[],
    );
}

#[test]
fn given_atomic_io_and_watch_capture_when_checked_then_keeps_distinct_purity_contracts() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(counter: &std::sync::atomic::AtomicUsize) {\n    counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { let content = std::fs::read(\"file\"); Some(current + 1) });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>, other: tokio::sync::mpsc::Sender<u32>) {\n    state.send_modify(|current| { let sent = other.try_send(*current); });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Sender;\nimpl Sender { fn send_modify<F>(&self, _: F) {} }\nfn run(state: Sender) { state.send_modify(|| { let content = std::fs::read(\"file\"); }); }",
        &[],
    );
}

#[test]
fn given_blocking_io_on_a_typed_std_receiver_in_watch_closure_when_checked_then_reports_only_the_io_closures()
 {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "use std::io::Write;\nfn write_file(state: tokio::sync::watch::Sender<u32>, mut file: std::fs::File) {\n    state.send_modify(|current| { let written = file.write_all(b\"x\"); *current += 1; });\n}\nfn write_stream(state: tokio::sync::watch::Sender<u32>, mut stream: std::net::TcpStream) {\n    state.send_if_modified(|current| { let flushed = stream.flush(); *current += 1; true });\n}\nfn run_command(state: tokio::sync::watch::Sender<u32>, mut command: std::process::Command) {\n    state.send_modify(|current| { let status = command.status(); *current += 1; });\n}\nfn read_input(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { let input = std::io::read_to_string(std::io::stdin()); *current += 1; });\n}\nfn write_buffer(state: tokio::sync::watch::Sender<u32>, mut buffer: Vec<u8>) {\n    state.send_modify(|current| { let written = buffer.write_all(b\"x\"); *current += 1; });\n}\nstruct Journal;\nimpl Journal { fn flush(&self) {} }\nfn flush_own(state: tokio::sync::watch::Sender<u32>, journal: Journal) {\n    state.send_modify(|current| { journal.flush(); *current += 1; });\n}\nfn build_error(state: tokio::sync::watch::Sender<String>) {\n    state.send_modify(|current| { *current = std::io::Error::other(\"x\").to_string(); });\n}",
        &[3, 6, 9, 12],
    );
}

#[test]
fn given_command_output_status_stream_write_all_and_io_copy_in_watch_closure_when_checked_then_reports_each()
 {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run_output(state: tokio::sync::watch::Sender<u32>, mut command: std::process::Command) {\n    state.send_modify(|current| { let output = command.output(); *current += 1; });\n}\nfn run_status(state: tokio::sync::watch::Sender<u32>, mut command: std::process::Command) {\n    state.send_modify(|current| { let status = command.status(); *current += 1; });\n}\nfn write_stream(state: tokio::sync::watch::Sender<u32>, mut stream: std::net::TcpStream) {\n    state.send_modify(|current| { let written = stream.write_all(b\"x\"); *current += 1; });\n}\nfn copy_all(state: tokio::sync::watch::Sender<u32>, mut left: std::fs::File, mut right: std::fs::File) {\n    state.send_modify(|current| { let copied = std::io::copy(&mut left, &mut right); *current += 1; });\n}",
        &[2, 5, 8, 11],
    );
}

#[test]
fn given_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(counter: &std::sync::atomic::AtomicUsize) {\n    let mut captured = 0;\n    counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured = current; Some(current + 1) });\n}\n",
        &[3],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>) {\n    let mut captured = None;\n    state.send_modify(|current| { captured = Some(*current); *current += 1; });\n}\n",
        &[],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&ImpureUpdateClosure, "ImpureUpdateClosure", "bad");
    assert_code_fixture(&ImpureUpdateClosure, "ImpureUpdateClosure", "good");
}

#[test]
fn given_impure_update_closure_captured_atomic_write_when_linted_then_reports_the_closure() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>, counter: std::sync::atomic::AtomicUsize) {\n    state.send_modify(|current| { counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed); *current += 1; });\n}\nstruct Ticks;\nimpl Ticks { fn fetch_add(&self, _: u32) {} }\nfn run_custom(state: tokio::sync::watch::Sender<u32>, ticks: Ticks) {\n    state.send_modify(|current| { ticks.fetch_add(1); *current += 1; });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(gate: std::sync::atomic::AtomicUsize, counter: std::sync::atomic::AtomicUsize) {\n    gate.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed); Some(current + 1) });\n}",
        &[2],
    );
}

#[test]
fn given_impure_update_closure_effect_in_macro_argument_when_linted_then_reports_the_closure() {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct Label { label: String }\nstruct Component { state: tokio::sync::watch::Sender<Label>, cache: parking_lot::Mutex<Vec<u8>> }\nimpl Component {\n    fn refresh(&self) {\n        self.state.send_modify(|current| {\n            current.label = format!(\"{}\", self.cache.lock().len());\n        });\n    }\n    fn refresh_pure(&self) {\n        self.state.send_modify(|current| {\n            current.label = format!(\"{}\", current.label.len());\n        });\n    }\n}",
        &[5],
    );
}

#[test]
fn given_impure_update_closure_effect_in_nested_closure_when_linted_then_reports_the_outer_closure()
{
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "struct State { ids: Vec<u32> }\nstruct Component { state: tokio::sync::watch::Sender<State>, seen: parking_lot::Mutex<Vec<u32>> }\nimpl Component {\n    fn refresh(&self) {\n        self.state.send_modify(|current| {\n            current.ids.retain(|id| self.seen.lock().contains(id));\n        });\n    }\n    fn refresh_pure(&self) {\n        self.state.send_modify(|current| {\n            current.ids.retain(|id| *id > 0);\n        });\n    }\n}",
        &[5],
    );
}

#[test]
fn given_fetch_update_closure_writing_a_capture_in_place_when_checked_then_reports_the_closure() {
    for statement in [
        "carried.take();",
        "carried.replace(1);",
        "carried.insert(1);",
        "carried.get_or_insert(1);",
        "carried.get_or_insert_with(|| 1);",
        "std::mem::take(&mut carried);",
        "std::mem::replace(&mut carried, None);",
    ] {
        assert_code_findings(
            &ImpureUpdateClosure,
            "ImpureUpdateClosure",
            &format!(
                "fn claim(counter: &std::sync::atomic::AtomicUsize, mut carried: Option<usize>) {{\n    let _updated = counter.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| {{\n        {statement}\n        Some(current + 1)\n    }});\n}}"
            ),
            &[2],
        );
    }
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn claim(counter: &std::sync::atomic::AtomicUsize, mut buffer: Vec<usize>) {\n    let _updated = counter.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| {\n        std::mem::take(&mut buffer);\n        Some(current + 1)\n    });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn claim(counter: &std::sync::atomic::AtomicUsize, carried: Option<usize>) {\n    let _updated = counter.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| {\n        let _present = carried.is_some();\n        Some(current + 1)\n    });\n}",
        &[],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn claim(state: tokio::sync::watch::Sender<usize>, mut carried: Option<usize>) {\n    state.send_modify(|current| {\n        carried.take();\n        *current += 1;\n    });\n}",
        &[],
    );
}

#[test]
fn given_watch_closure_building_a_net_value_without_io_when_checked_then_reports_only_the_file_read()
 {
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn socket(state: tokio::sync::watch::Sender<std::net::SocketAddr>, ip: std::net::IpAddr, port: u16) {\n    state.send_modify(|current| { *current = std::net::SocketAddr::new(ip, port); });\n}\nfn address(state: tokio::sync::watch::Sender<std::net::Ipv4Addr>) {\n    state.send_modify(|current| { *current = std::net::Ipv4Addr::new(127, 0, 0, 1); });\n}\nfn read(state: tokio::sync::watch::Sender<usize>) {\n    state.send_modify(|current| { *current = std::fs::read(\"file\").map_or(0, |bytes| bytes.len()); });\n}",
        &[8],
    );
}

#[test]
fn given_std_stdout_handle_in_watch_closure_when_checked_then_reports_handle_macro_write_and_flush()
{
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn run(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { let _handle = std::io::stdout(); *current += 1; });\n}",
        &[2],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "use std::io::Write;\nfn run(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { let written = writeln!(std::io::stdout(), \"x\"); *current += 1; });\n}",
        &[3],
    );
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "use std::io::Write;\nfn run(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { let flushed = std::io::stdout().flush(); *current += 1; });\n}",
        &[3],
    );
}

#[test]
fn given_thread_sleep_and_log_events_in_watch_closure_when_checked_then_reports_only_the_closures()
{
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "use tracing::warn;\nfn pause(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { std::thread::sleep(std::time::Duration::from_millis(1)); *current += 1; });\n}\nfn trace(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { tracing::debug!(\"bump\"); *current += 1; });\n}\nfn note(state: tokio::sync::watch::Sender<u32>) {\n    state.send_if_modified(|current| { log::info!(\"bump\"); *current += 1; true });\n}\nfn imported(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { warn!(\"bump\"); *current += 1; });\n}\nfn outside(state: tokio::sync::watch::Sender<u32>) {\n    tracing::info!(\"before\");\n    std::thread::sleep(std::time::Duration::from_millis(1));\n    state.send_modify(|current| { *current += 1; });\n}",
        &[3, 6, 9, 12],
    );
}

#[test]
fn given_fetch_update_closure_borrowing_a_destructured_local_when_checked_then_reports_only_capture()
 {
    for body in [
        "let Some(mut next) = step(current) else { return None; }; advance(&mut next); Some(next)",
        "if let Some(mut next) = step(current) { advance(&mut next); Some(next) } else { None }",
        "match step(current) { Some(mut next) => { advance(&mut next); Some(next) } None => None }",
        "let mut sum = 0; for (mut left, _) in [(current, 1)] { advance(&mut left); sum += left; } Some(sum)",
        "let Step { mut value } = Step { value: current }; advance(&mut value); Some(value)",
    ] {
        assert_code_findings(
            &ImpureUpdateClosure,
            "ImpureUpdateClosure",
            &format!(
                "struct Step {{ value: usize }}\nfn advance(_: &mut usize) {{}}\nfn pure(counter: &std::sync::atomic::AtomicUsize, step: fn(usize) -> Option<usize>) {{\n    let _updated = counter.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| {{\n        {body}\n    }});\n}}\nfn real(counter: &std::sync::atomic::AtomicUsize, mut captured: usize) {{\n    let _updated = counter.fetch_update(std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst, |current| {{\n        advance(&mut captured);\n        Some(current + 1)\n    }});\n}}"
            ),
            &[9],
        );
    }
}

#[test]
fn given_error_trace_and_event_macros_in_watch_closure_when_checked_then_reports_only_the_closures()
{
    assert_code_findings(
        &ImpureUpdateClosure,
        "ImpureUpdateClosure",
        "fn error(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { tracing::error!(\"bump\"); *current += 1; });\n}\nfn trace(state: tokio::sync::watch::Sender<u32>) {\n    state.send_modify(|current| { tracing::trace!(\"bump\"); *current += 1; });\n}\nfn event(state: tokio::sync::watch::Sender<u32>) {\n    state.send_if_modified(|current| { tracing::event!(tracing::Level::INFO, \"bump\"); *current += 1; true });\n}\nfn note(state: tokio::sync::watch::Sender<u32>) {\n    state.send_if_modified(|current| { log::error!(\"bump\"); *current += 1; true });\n}\nfn outside(state: tokio::sync::watch::Sender<u32>) {\n    tracing::error!(\"before\");\n    tracing::trace!(\"before\");\n    tracing::event!(tracing::Level::INFO, \"before\");\n    log::error!(\"before\");\n    state.send_modify(|current| { *current += 1; });\n}",
        &[2, 5, 8, 11],
    );
}
