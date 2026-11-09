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
#[test]
fn given_root_service_creation_forms_when_checked_then_literals_units_defaults_and_helpers_fail() {
    for expression in [
        "Service {}",
        "UnitService",
        "Service::default()",
        "build_service()",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::RootModuleConstruction,
            &format!(
                "struct Service {{}} struct UnitService; fn build_service() -> Service {{ Service {{}} }} mod di {{ use super::{{Service, UnitService, build_service}}; struct RootModule; impl RootModule {{ fn new() -> Self {{ let service = {expression}; Self }} }} }}"
            ),
            &[1],
        );
    }
}
#[test]
fn given_resolved_associated_factory_when_root_constructs_a_service_then_it_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::RootModuleConstruction,
        "struct Service; impl Service { fn open() -> Self { Self } } mod di { struct RootModule; impl RootModule { fn new() -> Self { let service = super::Service::open(); Self } } }",
        &[1],
    );
}
