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
fn given_parent_owning_a_child_without_lifecycle_when_checked_then_only_the_unforwarded_lifecycle_child_reports()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "mod di {\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); }\n struct ConfigModule;\n struct InvoiceModule;\n impl Lifecycle for InvoiceModule { fn on_enable(&self) {} fn on_disable(&self) {} }\n struct RootModule { config_module: ConfigModule, invoice_module: InvoiceModule }\n impl Lifecycle for RootModule { fn on_enable(&self) { self.invoice_module.on_enable(); } fn on_disable(&self) { self.invoice_module.on_disable(); } }\n struct StaleModule { invoice_module: InvoiceModule }\n impl Lifecycle for StaleModule { fn on_enable(&self) {} fn on_disable(&self) { self.invoice_module.on_disable(); } }\n}",
        &[8],
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
#[test]
fn given_lifecycle_paths_when_checked_then_every_owned_child_and_abort_must_run() {
    for body in [
        "",
        "if false { self.scope.abort(); }",
        "if flag { return; } self.scope.abort();",
        "if flag { self.scope.abort(); } else { self.scope.abort(); }",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleLifecycle,
            &format!(
                "struct TaskScope; trait Lifecycle {{ fn on_disable(&self); }} mod di {{ struct ParentModule {{ scope: super::TaskScope }} impl super::Lifecycle for ParentModule {{ fn on_disable(&self) {{ let flag = true; {body} }} }} }}"
            ),
            if body.contains("else") { &[] } else { &[1] },
        );
    }
    for method in ["on_enable", "on_disable", "on_reload"] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleLifecycle,
            &format!(
                "mod di {{ struct ParentModule; impl ParentModule {{ async fn {method}(&self) {{}} }} }}"
            ),
            &[1],
        );
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleLifecycle,
            &format!(
                "mod di {{ struct ChildModule; struct ParentModule {{ one: ChildModule, two: ChildModule }} impl ParentModule {{ fn {method}(&self) {{ self.one.{method}(); }} }} }}"
            ),
            &[1],
        );
    }
}

#[test]
fn given_dependency_order_when_forwarding_lifecycle_then_independent_field_order_does_not_matter() {
    for method in ["on_enable", "on_disable", "on_reload"] {
        for correct in [false, true] {
            let order = if correct == (method == "on_disable") {
                ["dependent", "provider"]
            } else {
                ["provider", "dependent"]
            };
            crate::test::module::util::assert_di_lines(
                &crate::module::rule::DiModuleLifecycle,
                &format!(
                    "mod di {{ trait Lifecycle {{ fn {method}(&self); }} struct ProviderModule; struct DependentModule; impl DependentModule {{ fn new(provider: &ProviderModule) -> Self {{ Self }} }} struct ParentModule {{ dependent: DependentModule, provider: ProviderModule }} impl Lifecycle for ParentModule {{ fn {method}(&self) {{ self.{}.{method}(); self.{}.{method}(); }} }} }}",
                    order[0], order[1]
                ),
                if correct { &[] } else { &[1] },
            );
        }
    }
}
#[test]
fn given_many_independent_conditionals_when_forwarding_then_equivalent_execution_states_are_merged()
{
    let branches = "if flag {}".repeat(30);
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        &format!(
            "trait Lifecycle {{ fn on_disable(&self); }} struct TaskScope; mod di {{ struct ParentModule {{ scope: super::TaskScope }} impl super::Lifecycle for ParentModule {{ fn on_disable(&self) {{ let flag = true; {branches} self.scope.abort(); }} }} }}"
        ),
        &[],
    );
}
#[test]
fn given_inherited_lifecycle_defaults_when_checked_then_only_child_work_requires_explicit_forwarding()
 {
    for child in ["", "fn on_reload(&self) {}"] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleLifecycle,
            &format!(
                "trait Lifecycle {{ fn on_reload(&self) {{}} }} mod di {{ struct ChildModule; impl super::Lifecycle for ChildModule {{ {child} }} struct ParentModule {{ child: ChildModule }} impl super::Lifecycle for ParentModule {{}} }}"
            ),
            if child.is_empty() { &[] } else { &[1] },
        );
    }
}
#[test]
fn given_many_distinct_branch_histories_when_forwarding_then_pairwise_flow_states_remain_bounded() {
    let branches = "if flag { self.one.on_enable(); } else { self.two.on_enable(); }".repeat(30);
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        &format!(
            "trait Lifecycle {{ fn on_enable(&self); }} mod di {{ struct ChildModule; struct ParentModule {{ one: ChildModule, two: ChildModule }} impl super::Lifecycle for ParentModule {{ fn on_enable(&self) {{ let flag = true; {branches} self.one.on_enable(); self.two.on_enable(); }} }} }}"
        ),
        &[],
    );
}
#[test]
fn given_nested_branch_projections_when_forwarding_then_repeated_subtree_states_are_memoized() {
    let mut body = "self.one.on_enable(); self.two.on_enable();".to_owned();
    for _ in 0..25 {
        body = format!(
            "if flag {{ self.one.on_enable(); }} else {{ self.two.on_enable(); }} {{ {body} }}"
        );
    }
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        &format!(
            "trait Lifecycle {{ fn on_enable(&self); }} mod di {{ struct ChildModule; struct ParentModule {{ one: ChildModule, two: ChildModule }} impl super::Lifecycle for ParentModule {{ fn on_enable(&self) {{ let flag = true; {body} }} }} }}"
        ),
        &[],
    );
}

#[test]
fn given_own_task_started_in_on_enable_when_checked_then_only_received_module_lifecycle_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nstruct WarmupUseCase;\nmod di {\n use super::{TaskScope, WarmupUseCase};\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); }\n struct InvoiceModule { warmup_use_case: std::sync::Arc<WarmupUseCase>, task_scope: std::sync::Arc<TaskScope> }\n impl Lifecycle for InvoiceModule { fn on_enable(&self) { let warmup_use_case = std::sync::Arc::clone(&self.warmup_use_case); self.task_scope.spawn(async move { warmup_use_case.invoke().await; }); } fn on_disable(&self) {} }\n struct ChildModule;\n struct BorrowingModule<'a> { child: &'a ChildModule }\n impl Lifecycle for BorrowingModule<'_> { fn on_enable(&self) { self.child.on_enable(); } fn on_disable(&self) {} }\n}",
        &[9],
    );
}

#[test]
fn given_own_task_scope_reloaded_in_on_reload_when_checked_then_it_is_accepted() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); fn on_reload(&self); }\n struct InvoiceModule { task_scope: TaskScope }\n impl Lifecycle for InvoiceModule { fn on_enable(&self) {} fn on_disable(&self) { self.task_scope.abort(); } fn on_reload(&self) { self.task_scope.reload(); } }\n}",
        &[],
    );
}

#[test]
fn given_reload_of_a_received_scope_or_with_a_lifecycle_call_in_its_argument_when_checked_then_it_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_enable(&self); fn on_reload(&self); }\n struct InvoiceModule { task_scope: std::sync::Arc<TaskScope> }\n impl Lifecycle for InvoiceModule { fn on_enable(&self) {} fn on_reload(&self) { self.task_scope.reload(); } }\n}",
        &[5],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nstruct Other;\nmod di {\n use super::{Other, TaskScope};\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); fn on_reload(&self); }\n struct InvoiceModule { task_scope: TaskScope, other: std::sync::Arc<Other> }\n impl Lifecycle for InvoiceModule { fn on_enable(&self) {} fn on_disable(&self) { self.task_scope.abort(); } fn on_reload(&self) { self.task_scope.reload(self.other.compute()); } }\n}",
        &[6],
    );
}

#[test]
fn given_task_scope_reloaded_outside_on_reload_or_not_owned_when_checked_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); fn on_reload(&self); }\n struct InvoiceModule { task_scope: TaskScope }\n impl Lifecycle for InvoiceModule { fn on_enable(&self) { self.task_scope.reload(); } fn on_disable(&self) { self.task_scope.abort(); } fn on_reload(&self) {} }\n}",
        &[5],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nstruct CoreModule { task_scope: std::sync::Arc<TaskScope> }\nmod di {\n use super::{CoreModule, TaskScope};\n trait Lifecycle { fn on_enable(&self); fn on_reload(&self); }\n struct BorrowingModule<'a> { core: &'a CoreModule }\n impl Lifecycle for BorrowingModule<'_> { fn on_enable(&self) {} fn on_reload(&self) { self.core.task_scope.reload(); } }\n}",
        &[6],
    );
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct WarmupUseCase;\nmod di {\n use super::WarmupUseCase;\n trait Lifecycle { fn on_enable(&self); fn on_reload(&self); }\n struct InvoiceModule { warmup_use_case: std::sync::Arc<WarmupUseCase> }\n impl Lifecycle for InvoiceModule { fn on_enable(&self) {} fn on_reload(&self) { self.warmup_use_case.reload(); } }\n}",
        &[5],
    );
}

#[test]
fn given_non_arc_task_scope_fields_when_checked_then_only_arc_is_accepted() {
    crate::test::module::util::assert_di_lines(
        &crate::module::rule::DiModuleLifecycle,
        "struct TaskScope;\nstruct BoxedWorker { scope: Box<TaskScope> }\nstruct SharedWorker { scope: std::sync::Arc<TaskScope> }\nstruct RcWorker { scope: std::rc::Rc<TaskScope> }",
        &[2, 4],
    );
}

#[test]
fn given_foreign_call_in_lifecycle_macro_argument_when_checked_then_child_work_in_on_enable_is_reported()
 {
    for statement in [
        "tracing::info!(count = self.child.warm_up(), \"warmed up\");",
        "let _ = self.child.warm_up();",
    ] {
        crate::test::module::util::assert_di_lines(
            &crate::module::rule::DiModuleLifecycle,
            &format!(
                "mod di {{\n struct ChildModule;\n struct ParentModule {{ child: ChildModule }}\n trait Lifecycle {{ fn on_enable(&self); fn on_disable(&self); }}\n impl Lifecycle for ParentModule {{ fn on_enable(&self) {{ self.child.on_enable(); {statement} }} fn on_disable(&self) {{ self.child.on_disable(); }} }}\n}}"
            ),
            &[3],
        );
    }
}
