use crate::runtime::rule::TimePrimitive;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_shadowed_instant_patterns_when_logged_then_uses_the_nearest_binding() {
    for body in [
        "let log_value = |start: u64| { tracing::info!(value = start); }; log_value(1);",
        "for start in [1u64] { tracing::info!(value = start); }",
        "if let Some(start) = Some(1u64) { tracing::info!(value = start); }",
        "match Some(1u64) { Some(start) => tracing::info!(value = start), None => {} }",
        "match Some(1u64) { Some(start) if start > 0 => tracing::info!(value = start), _ => {} }",
    ] {
        assert_code_findings(
            &TimePrimitive,
            "TimePrimitive",
            &format!("fn run() {{ let start = std::time::Instant::now(); {body} }}"),
            &[],
        );
        assert_code_findings(
            &TimePrimitive,
            "TimePrimitive",
            &format!(
                "fn run() {{ let start = std::time::Instant::now(); {body}\n tracing::info!(value = ?start);\n}}"
            ),
            &[2],
        );
    }
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "fn run() { let start = std::time::Instant::now();\n let start = { tracing::info!(value = ?start); 1u64 }; tracing::info!(value = start);\n}",
        &[2],
    );
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "fn run() { let start = std::time::Instant::now(); let log_value = || { tracing::info!(value = ?start); }; log_value(); }",
        &[1],
    );
}

#[test]
fn given_elapsed_duration_log_when_checked_then_checks_logged_value_instead_of_receiver_token() {
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "fn run() { let start = std::time::Instant::now(); tracing::info!(elapsed = ?start.elapsed()); log::info!(\"{:?}\", start.elapsed()); }",
        &[],
    );
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "fn run() { let start = std::time::Instant::now();\n tracing::info!(instant = ?start);\n log::info!(\"{:?}\", start);\n}",
        &[2, 3],
    );
}

#[test]
fn given_float_text_constants_and_local_instant_when_checked_then_preserves_local_elapsed_use() {
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "const DELAY_SECONDS: f64 = 0.5;\nconst WAIT_MS: &str = \"400\";\nfn run(sender: tokio::sync::mpsc::Sender<std::time::Instant>) { let now = std::time::Instant::now();\n tracing::info!(\"{now:?}\");\n sender.try_send(now);\n}",
        &[1, 2, 4, 5],
    );
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "fn run() { let started = std::time::Instant::now(); let duration = started.elapsed(); tracing::info!(?duration); }\nconst WAIT: std::time::Duration = std::time::Duration::from_millis(400);",
        &[],
    );
}

#[test]
fn given_normative_runtime_counterexamples_when_checked_then_matches_exact_node_anchors() {
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "use std::time::Instant as Tick;\nstruct Stored { at: Tick }\nfn now() -> Tick { loop {} }\nconst DELAY_MS: u64 = 40;\n",
        &[2, 3, 4],
    );
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "use std::time::Duration;\nfn decode_timeout(timeout_ms: u64) -> Duration { Duration::from_millis(timeout_ms) }\nconst DELAY: Duration = Duration::from_millis(40);\n",
        &[],
    );
}

#[test]
fn given_minute_and_hour_unit_constants_when_building_duration_then_reports_primitive_constants() {
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "use std::time::Duration;\nconst WAIT_MINUTES: u64 = 5;\nconst WAIT_HOURS: u64 = 1;\nconst WAIT: Duration = Duration::from_secs(60);\nfn pause() -> Duration { Duration::from_secs(WAIT_MINUTES * 60 + WAIT_HOURS * 3600) }",
        &[2, 3],
    );
}

#[test]
fn given_static_and_trait_constants_with_unit_suffix_when_checked_then_reports_both() {
    assert_code_findings(
        &TimePrimitive,
        "TimePrimitive",
        "pub(crate) static CARD_IN_MS: u64 = 400;\npub(crate) trait Timing {\n    const FADE_MS: u64;\n}\npub(crate) const SLIDE_MS: u64 = 1;",
        &[1, 3, 5],
    );
}

#[test]
fn given_persistent_runtime_cases_when_checked_then_matches_exact_finding_anchors() {
    assert_code_fixture(&TimePrimitive, "TimePrimitive", "bad");
    assert_code_fixture(&TimePrimitive, "TimePrimitive", "good");
}
