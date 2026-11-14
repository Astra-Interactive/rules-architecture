use crate::test::declaration::util::FindingMessage;
use crate::test::declaration::util::list_finding_messages;

#[test]
fn given_clock_constructions_when_checked_then_di_and_test_tree_are_exempt() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "mod system { pub struct SystemClock; }\n fn build() { let clock = system::SystemClock; }\n mod di { fn build() { let clock = crate::system::SystemClock; } }\n #[cfg(test)] mod tests { fn build() { let clock = crate::system::SystemClock; } }",
        &[2],
    );
}

#[test]
fn given_clock_signatures_when_checked_then_exact_standard_time_and_shared_receiver_are_required() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "mod system { struct SystemClock; } trait Clock { fn now(&self) -> std::time::SystemTime; }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "struct SystemClock;\ntrait Clock { fn now(&mut self) -> std::time::SystemTime; }",
        &[1, 2],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "trait Clock { fn now(&self) -> Result<std::time::SystemTime, ()>; }",
        &[1],
    );
}

fn list_clock_messages(source: &str) -> Vec<FindingMessage> {
    let context = crate::test::module::util::read_di_context(source);
    list_finding_messages(&crate::check::api::Rule::check(
        &crate::module::rule::SystemClockOutsideDi,
        &context,
    ))
}

#[test]
fn given_misplaced_system_clock_when_checked_then_the_message_names_the_system_module() {
    assert_eq!(
        vec![FindingMessage {
            line: 1,
            message: "SystemClock is declared in a system module (time-api.rs.md)".to_owned(),
        }],
        list_clock_messages("mod clock { pub struct SystemClock; }")
    );
}

#[test]
fn given_clock_with_another_signature_when_checked_then_the_message_names_the_trait_signature() {
    assert_eq!(
        vec![FindingMessage {
            line: 1,
            message:
                "the Clock trait declares exactly fn now(&self) -> SystemTime (time-api.rs.md)"
                    .to_owned(),
        }],
        list_clock_messages("trait Clock { fn now(&self) -> std::time::Instant; }")
    );
}

#[test]
fn given_di_alias_shape_when_checked_then_preserves_defined_exceptions() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "trait Clock { fn now(&self) -> &std::time::SystemTime; }",
        &[1],
    );
}

#[test]
fn given_second_wall_clock_reader_under_expect_when_checked_then_only_system_clock_may_call_now() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "trait Clock { fn now(&self) -> std::time::SystemTime; }\nmod system {\n pub struct SystemClock;\n impl super::Clock for SystemClock {\n  #[expect(clippy::disallowed_methods, reason = \"the single wall clock reader\")]\n  fn now(&self) -> std::time::SystemTime { std::time::SystemTime::now() }\n }\n}\nstruct NtpClock;\nimpl Clock for NtpClock {\n #[expect(clippy::disallowed_methods, reason = \"reads the wall clock\")]\n fn now(&self) -> std::time::SystemTime { std::time::SystemTime::now() }\n}\nmod usecase {\n #[expect(clippy::disallowed_methods, reason = \"reads the wall clock\")]\n pub fn deadline() -> std::time::SystemTime { std::time::SystemTime::now() }\n}",
        &[12, 16],
    );
}

#[test]
fn given_time_crate_clock_reads_and_epoch_elapsed_outside_system_clock_when_checked_then_each_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::SystemClockOutsideDi,
        "trait Clock { fn now(&self) -> std::time::SystemTime; }\nmod system {\n pub struct SystemClock;\n impl super::Clock for SystemClock {\n  #[expect(clippy::disallowed_methods, reason = \"the single wall clock reader\")]\n  fn now(&self) -> std::time::SystemTime { std::time::SystemTime::now() }\n }\n}\nmod usecase {\n #[expect(clippy::disallowed_methods, reason = \"reads the wall clock\")]\n pub fn stamp() -> chrono::DateTime<chrono::Utc> { chrono::Utc::now() }\n #[expect(clippy::disallowed_methods, reason = \"reads the wall clock\")]\n pub fn uptime() -> std::time::Duration { std::time::UNIX_EPOCH.elapsed().unwrap_or_default() }\n pub fn tick() -> std::time::Instant { std::time::Instant::now() }\n}",
        &[11, 13],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::SystemClockOutsideDi,
            "SystemClockOutsideDi",
            half,
        );
    }
}
