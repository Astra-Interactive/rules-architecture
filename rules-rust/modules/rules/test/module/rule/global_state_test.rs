#[test]
fn given_logging_installations_when_checked_then_entry_main_is_the_only_installation_boundary() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "use tracing::subscriber::set_global_default as install;\nfn configure() { install(todo!()); }\nfn helper() { tracing_subscriber::fmt().init(); }",
        &[2, 3],
    );
    let mut context = crate::test::module::util::read_di_context(
        "fn main() { tracing_subscriber::fmt().init(); } ",
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    assert!(crate::check::api::Rule::check(&crate::module::rule::GlobalState, &context).is_empty());
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "struct Builder; impl Builder { fn init(&self) {} } fn configure(builder: Builder) { builder.init(); }",
        &[],
    );
}

#[test]
fn given_global_state_alias_and_unrelated_mutex_when_checked_then_only_resolved_standard_state_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "type Shared = std::sync::Mutex<u64>;\nstatic BAD: Shared = todo!();\nmod local { pub struct Mutex; } static GOOD: local::Mutex = local::Mutex;",
        &[2],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "static VALUE: u64 = 1;",
        &[],
    );
}

#[test]
fn given_immutable_static_service_when_checked_then_it_is_reported_and_plain_number_is_not() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "mod api { pub trait Client { fn read(&self) -> std::io::Result<String>; } }\nstruct FileClient;\nimpl crate::api::Client for FileClient { fn read(&self) -> std::io::Result<String> { std::fs::read_to_string(\"input\") } }\nstatic CLIENT: FileClient = FileClient;\nstatic LIMIT: u64 = 1;",
        &[4],
    );
}

#[test]
fn given_static_mut_when_checked_then_it_is_reported_and_plain_static_is_not() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "pub static mut COUNTER: u64 = 0;\nstatic LIMIT: u64 = 1;",
        &[1],
    );
}

#[test]
fn given_function_local_statics_when_checked_then_each_is_reported_once() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "use std::sync::atomic::{AtomicU64, Ordering};\nstatic GLOBAL: std::sync::Mutex<u64> = std::sync::Mutex::new(0);\nfn next_id() -> u64 {\n    static NEXT: AtomicU64 = AtomicU64::new(0);\n    NEXT.fetch_add(1, Ordering::Relaxed)\n}\nfn instance() -> &'static String {\n    static CELL: std::sync::OnceLock<String> = std::sync::OnceLock::new();\n    CELL.get_or_init(String::new)\n}",
        &[2, 4, 8],
    );
}

#[test]
fn given_logging_install_spellings_outside_main_when_checked_then_each_is_reported() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::GlobalState,
        "fn first() { tracing_subscriber::fmt::init(); }\nfn second() { tracing_subscriber::fmt().try_init(); }\nfn third() { log::set_logger(todo!()); }\nfn helper() { tracing_subscriber::fmt().init(); }",
        &[1, 2, 3, 4],
    );
}

#[test]
fn given_thread_local_state_when_checked_then_reports_the_static_and_not_constant_data() {
    let context = crate::test::module::util::read_placement_context(&[
        ("src/lib.rs", "mod state;"),
        (
            "src/state.rs",
            "thread_local! {\n    static CURRENT_USER: std::cell::RefCell<Option<u32>> = const { std::cell::RefCell::new(None) };\n}\nthread_local! {\n    static DEPTH: u32 = 0;\n}",
        ),
    ]);
    crate::test::module::util::assert_placement_findings(
        &crate::module::rule::GlobalState,
        "GlobalState",
        &context,
        &[("src/state.rs", 2, "Node")],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::GlobalState,
            "GlobalState",
            half,
        );
    }
}
