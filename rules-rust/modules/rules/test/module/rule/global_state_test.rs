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
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::GlobalState,
            "GlobalState",
            half,
        );
    }
}
