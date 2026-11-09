#[test]
fn given_module_parameter_cycles_when_checked_then_incompatible_cfg_cycles_are_accepted() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "mod di {\n struct OneModule; impl OneModule { fn new(two: &TwoModule) -> Self { Self } }\n struct TwoModule; impl TwoModule { fn new(one: &OneModule) -> Self { Self } }\n}",
        &[2, 3],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "mod di {\n struct OneModule; #[cfg(unix)] impl OneModule { fn new(two: &TwoModule) -> Self { Self } }\n struct TwoModule; #[cfg(not(unix))] impl TwoModule { fn new(one: &OneModule) -> Self { Self } }\n}",
        &[],
    );
}

#[test]
fn given_owned_children_and_received_module_when_lifecycle_checked_then_only_owned_lifecycle_is_allowed()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "mod di {\n struct ChildModule; struct ParentModule { child: ChildModule }\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); }\n impl Lifecycle for ParentModule { fn on_enable(&self) { self.child.on_enable(); } fn on_disable(&self) { self.child.on_disable(); } }\n impl ParentModule { fn new(child: &ChildModule) -> Self { todo!() } }\n}",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "mod di {\n struct ChildModule;\n struct ParentModule<'a> { child: &'a ChildModule }\n impl ParentModule<'_> { fn start(&self) {} fn on_disable(&self) { self.child.on_disable(); } }\n}",
        &[3],
    );
}

#[test]
fn given_scopes_and_shutdown_waits_when_checked_then_abort_is_required_and_shutdown_is_synchronous()
{
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope; mod di { use super::TaskScope;\n struct ParentModule { scope: TaskScope }\n trait Lifecycle { fn on_disable(&self); } impl Lifecycle for ParentModule { fn on_disable(&self) { self.scope.abort(); } }\n}",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope; mod di { use super::TaskScope;\n struct ParentModule { scope: TaskScope }\n impl ParentModule { async fn on_disable(&self) { self.scope.join().await; } }\n}",
        &[2],
    );
}

#[test]
fn given_di_alias_shape_when_checked_then_preserves_defined_exceptions() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "mod di { struct ChildModule; type Borrowed<'a> = &'a ChildModule;\nstruct OwnerModule<'a> { child: Borrowed<'a> } }",
        &[2],
    );
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::DiModuleLifecycle,
            "DiModuleLifecycle",
            half,
        );
    }
}
