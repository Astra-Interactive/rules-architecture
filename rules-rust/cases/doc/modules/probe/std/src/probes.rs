use std::io::Write;
use std::sync::{Mutex, PoisonError, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

pub(crate) fn probe_unwrap_used(value: Option<u8>) -> u8 {
    value.unwrap()
}

pub(crate) fn probe_expect_used(value: Option<u8>) -> u8 {
    value.expect("probe")
}

pub(crate) fn probe_print_stdout() {
    println!("probe");
}

pub(crate) fn probe_print_stderr() {
    eprintln!("probe");
}

pub(crate) fn probe_dbg_macro(value: u8) -> u8 {
    dbg!(value)
}

#[allow(dead_code)]
pub(crate) fn probe_allow_attributes() {}

#[expect(dead_code)]
fn probe_allow_attributes_without_reason() {}

pub(crate) fn probe_let_underscore_must_use() {
    let _ = "1".parse::<u8>();
}

pub(crate) fn probe_undocumented_unsafe_blocks(values: &[u8]) -> u8 {
    unsafe { *values.get_unchecked(0) }
}

unsafe fn probe_check_private_items(value: *const u8) -> u8 {
    // SAFETY: probe only; the caller passes a valid pointer.
    unsafe { *value }
}

pub(crate) fn probe_redundant_closure_for_method_calls(text: &str) -> Vec<&str> {
    text.lines().map(|line| line.trim()).collect()
}

pub(crate) fn probe_std_mutex(lock: &Mutex<u8>) -> bool {
    lock.try_lock().is_ok()
}

pub(crate) fn probe_std_rw_lock(lock: &RwLock<u8>) -> bool {
    lock.try_read().is_ok()
}

pub(crate) fn probe_poison_error(error: PoisonError<u8>) -> u8 {
    PoisonError::into_inner(error)
}

pub(crate) fn probe_thread_spawn() -> bool {
    let handle = std::thread::spawn(|| 1);
    handle.join().is_ok()
}

pub(crate) fn probe_thread_builder_spawn() -> std::io::Result<bool> {
    let handle = std::thread::Builder::new().spawn(|| 1)?;
    Ok(handle.join().is_ok())
}

pub(crate) fn probe_io_stdout() -> std::io::Result<()> {
    std::io::stdout().write_all(b"probe")
}

pub(crate) fn probe_io_stderr() -> std::io::Result<()> {
    std::io::stderr().write_all(b"probe")
}

pub(crate) fn probe_system_time_now() -> SystemTime {
    SystemTime::now()
}

pub(crate) fn probe_system_time_elapsed() -> bool {
    UNIX_EPOCH.elapsed().is_ok()
}

pub(crate) async fn probe_await_holding_watch_ref(receiver: &watch::Receiver<u8>) -> u8 {
    let value = receiver.borrow();
    tokio::task::yield_now().await;
    *value
}

pub(crate) fn probe_watch_sender_send(sender: &watch::Sender<u8>) -> bool {
    sender.send(1).is_ok()
}

pub fn probe_unreachable_pub() {}

#[cfg(klibs_rule)]
pub(crate) fn probe_unexpected_cfgs() {}

#[cfg_attr(
    klibs_rules,
    expect(
        AnonymousTuple,
        reason = "probe: the linter's exception compiles silently"
    )
)]
pub(crate) fn probe_silent_klibs_rules_cfg() {}

macro_rules! declare_probe_ids {
    ($($name:ident),*) => {
        $(pub(crate) struct $name;)*
    };
}

#[cfg_attr(
    klibs_rules,
    expect(
        Unanalysable,
        reason = "probe: an item macro call takes its own expectation"
    )
)]
declare_probe_ids!(ProbeOrderId, ProbeInvoiceId);

pub(crate) fn probe_excessive_nesting(values: &[u8]) -> u8 {
    let mut found = 0;
    for first in values {
        for second in values {
            for third in values {
                for fourth in values {
                    for fifth in values {
                        found = found.max(first ^ second ^ third ^ fourth ^ fifth);
                    }
                }
            }
        }
    }
    found
}

pub(crate) fn probe_too_many_lines() -> u32 {
    let mut total = 0;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total += 1;
    total
}

#[cfg(test)]
mod tests {
    #[test]
    fn probe_silent_allow_unwrap_in_tests() {
        let parsed = "1".parse::<u8>();
        assert_eq!(parsed.unwrap(), 1);
    }

    #[test]
    fn probe_silent_allow_expect_in_tests() {
        let parsed = "1".parse::<u8>();
        assert_eq!(parsed.expect("a test may expect"), 1);
    }
}
