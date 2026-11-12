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

#[test]
fn given_service_built_by_a_method_of_a_root_parameter_when_root_constructs_then_it_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::RootModuleConstruction,
        "struct Settings { url: String } struct Pool { url: String }\nimpl Settings { fn build_pool(&self) -> Pool { Pool { url: self.url.clone() } } fn url(&self) -> String { self.url.clone() } }\nmod di { use super::Settings; struct CoreModule; impl CoreModule { fn new(url: String) -> Self { Self } }\nstruct RootModule; impl RootModule { fn new(settings: Settings) -> Self {\nlet pool = settings.build_pool();\nlet core_module = CoreModule::new(settings.url());\nSelf } } }",
        &[5],
    );
}

#[test]
fn given_dependency_crate_client_built_under_non_constructor_name_when_root_constructs_then_it_is_rejected()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::RootModuleConstruction,
        "mod di {\n struct RootModule; impl RootModule { fn new() -> Self {\n let client = vendor::Client::builder().build();\n let pool = vendor::Pool::new();\n Self } }\n}",
        &[3, 4],
    );
}

#[test]
fn given_service_built_inside_a_macro_argument_when_root_constructs_then_it_is_rejected() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::RootModuleConstruction,
        "struct Service; impl Service { fn open() -> Self { Self } }\nmod di { struct RootModule; impl RootModule { fn new() -> Self {\n let services = vec![super::Service::open()];\n let sizes = vec![1, 2];\n Self } } }",
        &[3],
    );
}
