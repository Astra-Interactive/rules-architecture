#[test]
fn given_root_constructors_when_checked_then_only_module_construction_is_allowed() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::RootModuleConstruction,
        "mod di { struct ChildModule; impl ChildModule { fn new() -> Self { Self } } struct RootModule; impl RootModule { fn new() -> Self { let child = ChildModule::new(); Self } } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::RootModuleConstruction,
        "mod di { struct Service; impl Service { fn new() -> Self { Self } }\n struct RootModule; impl RootModule { fn new() -> Self { let service = Service::new(); Self } } }",
        &[2],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::RootModuleConstruction,
            "RootModuleConstruction",
            half,
        );
    }
}
