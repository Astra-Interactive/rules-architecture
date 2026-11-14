use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::DiModuleLifecycle;
use crate::source::syn::CrateIndexBuilder;

const HANDED_SCOPE_MESSAGE: &str = "A type that was handed a scope only spawns onto it: `abort` and `reload` belong to the scope's owner (async.rs.md, Tasks)";

fn list_lifecycle_messages(source: &str) -> Vec<String> {
    let context = crate::test::module::util::read_di_context(source);
    DiModuleLifecycle
        .check(&context)
        .iter()
        .map(|finding| finding.finding.message.clone())
        .collect()
}

#[test]
fn given_private_method_of_the_module_called_in_on_reload_when_checked_then_the_module_is_reported()
{
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "mod di {\n trait Lifecycle { fn on_reload(&self); }\n struct ListenerModule;\n impl Lifecycle for ListenerModule { fn on_reload(&self) {} }\n struct ConfigModule { listener: ListenerModule }\n impl Lifecycle for ConfigModule { fn on_reload(&self) { self.reload_configuration(); self.listener.on_reload(); } }\n impl ConfigModule { fn reload_configuration(&self) {} }\n}",
        &[5],
    );
}

#[test]
fn given_private_method_that_only_forwards_when_called_from_a_lifecycle_method_then_the_module_is_reported()
 {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "mod di {\n trait Lifecycle { fn on_reload(&self); }\n struct ListenerModule;\n impl Lifecycle for ListenerModule { fn on_reload(&self) {} }\n struct ConfigModule { listener: ListenerModule }\n impl Lifecycle for ConfigModule { fn on_reload(&self) { self.forward_reload(); } }\n impl ConfigModule { fn forward_reload(&self) { self.listener.on_reload(); } }\n}",
        &[5],
    );
}

#[test]
fn given_configuration_reload_as_a_job_of_the_owned_scope_when_checked_then_nothing_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct Job;\nmod di {\n use super::{Job, TaskScope};\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); fn on_reload(&self); }\n struct CoreModule { task_scope: TaskScope, job: std::sync::Arc<Job> }\n impl Lifecycle for CoreModule { fn on_enable(&self) { self.task_scope.spawn(std::sync::Arc::clone(&self.job)); } fn on_disable(&self) { self.task_scope.abort(); } fn on_reload(&self) { self.task_scope.reload(); } }\n}",
        &[],
    );
}

#[test]
fn given_module_that_spawns_onto_a_handed_scope_in_on_enable_when_checked_then_nothing_is_reported()
{
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct Job;\nmod di {\n use super::{Job, TaskScope};\n trait Lifecycle { fn on_enable(&self); }\n struct FeatureModule { task_scope: std::sync::Arc<TaskScope>, job: std::sync::Arc<Job> }\n impl Lifecycle for FeatureModule { fn on_enable(&self) { self.task_scope.spawn(std::sync::Arc::clone(&self.job)); } }\n}",
        &[],
    );
}

#[test]
fn given_module_that_aborts_a_handed_scope_in_on_disable_when_checked_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_disable(&self); }\n struct FeatureModule { task_scope: std::sync::Arc<TaskScope> }\n impl Lifecycle for FeatureModule { fn on_disable(&self) { self.task_scope.abort(); } }\n}",
        &[5],
    );
}

#[test]
fn given_technology_module_that_blocks_on_its_stored_runtime_handle_in_on_disable_when_checked_then_nothing_is_reported()
 {
    for role in [Role::Technology, Role::Platform, Role::CorePlatform] {
        assert_eq!(
            list_block_on_lines("self.handle.block_on(self.pool.close());", Some(role)),
            Vec::<u32>::new(),
            "{role:?}"
        );
    }
}

#[test]
fn given_block_on_in_a_module_that_is_not_a_technology_when_checked_then_it_is_reported() {
    for role in [None, Some(Role::Context), Some(Role::Api)] {
        assert_eq!(
            list_block_on_lines("self.handle.block_on(self.pool.close());", role),
            [8],
            "{role:?}"
        );
    }
}

#[test]
fn given_block_on_with_anything_but_one_call_on_an_own_field_when_checked_then_it_is_reported() {
    for body in [
        "self.handle.block_on(async { self.pool.close().await; });",
        "self.handle.block_on(self.pool.close().finish());",
        "self.handle.block_on(close_pool());",
        "self.handle.block_on(self.pool.close(self.pool.timeout()));",
        "self.other.block_on(self.pool.close());",
    ] {
        assert_eq!(
            list_block_on_lines(body, Some(Role::Technology)),
            [8],
            "{body}"
        );
    }
}

fn list_block_on_lines(body: &str, role: Option<Role>) -> Vec<u32> {
    let source = format!(
        "fn close_pool() {{}}\nstruct Pool;\nstruct Other;\nmod di {{\n use super::{{Other, Pool}};\n use tokio::runtime::Handle;\n trait Lifecycle {{ fn on_disable(&self); }}\n struct PoolModule {{ handle: Handle, pool: Pool, other: Other }}\n impl Lifecycle for PoolModule {{ fn on_disable(&self) {{ {body} }} }}\n}}"
    );
    let mut context = crate::test::module::util::read_di_context(&source);
    context.project_config.crates[0].role = role;
    DiModuleLifecycle
        .check(&context)
        .iter()
        .filter_map(|finding| finding.finding.location.line)
        .collect()
}

#[test]
fn given_block_on_outside_on_disable_when_checked_then_it_is_reported() {
    let mut context = crate::test::module::util::read_di_context(
        "struct Pool;\nmod di {\n use super::Pool;\n use tokio::runtime::Handle;\n trait Lifecycle { fn on_enable(&self); }\n struct PoolModule { handle: Handle, pool: Pool }\n impl Lifecycle for PoolModule { fn on_enable(&self) { self.handle.block_on(self.pool.close()); } }\n}",
    );
    context.project_config.crates[0].role = Some(Role::Technology);
    let lines: Vec<u32> = DiModuleLifecycle
        .check(&context)
        .iter()
        .filter_map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(lines, [6]);
}

#[test]
fn given_module_that_builds_a_shared_scope_in_its_new_when_checked_then_it_owns_it() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); fn on_reload(&self); }\n struct SharedModule { task_scope: std::sync::Arc<TaskScope> }\n impl SharedModule { fn new() -> Self { Self { task_scope: std::sync::Arc::new(TaskScope::new()) } } }\n impl Lifecycle for SharedModule { fn on_enable(&self) {} fn on_disable(&self) { self.task_scope.abort(); } fn on_reload(&self) { self.task_scope.reload(); } }\n}",
        &[],
    );
}

#[test]
fn given_module_that_builds_a_shared_scope_and_never_aborts_it_when_checked_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_enable(&self); fn on_disable(&self); }\n struct SharedModule { task_scope: std::sync::Arc<TaskScope> }\n impl SharedModule { fn new() -> Self { Self { task_scope: std::sync::Arc::new(TaskScope::new()) } } }\n impl Lifecycle for SharedModule { fn on_enable(&self) {} fn on_disable(&self) {} }\n}",
        &[5],
    );
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n struct SharedModule { task_scope: std::sync::Arc<TaskScope> }\n impl SharedModule { fn new() -> Self { Self { task_scope: std::sync::Arc::new(TaskScope::new()) } } }\n}",
        &[4],
    );
}

fn list_owner_module_lines(initializer: &str, preamble: &str) -> Vec<u32> {
    let source = format!(
        "struct TaskScope;\nmod di {{\n use super::TaskScope;\n trait Lifecycle {{ fn on_disable(&self); }}\n struct SharedModule {{ task_scope: std::sync::Arc<TaskScope> }}\n impl SharedModule {{ fn new() -> Self {{ {preamble} Self {{ task_scope: {initializer} }} }} }}\n impl Lifecycle for SharedModule {{ fn on_disable(&self) {{ self.task_scope.abort(); }} }}\n}}"
    );
    let context = crate::test::module::util::read_di_context(&source);
    DiModuleLifecycle
        .check(&context)
        .iter()
        .filter_map(|finding| finding.finding.location.line)
        .collect()
}

#[test]
fn given_shared_scope_built_by_default_or_by_a_local_when_checked_then_the_module_owns_it() {
    for (initializer, preamble) in [
        ("std::sync::Arc::new(TaskScope::default())", ""),
        ("std::sync::Arc::default()", ""),
        (
            "task_scope",
            "let task_scope = std::sync::Arc::new(TaskScope::new());",
        ),
    ] {
        assert_eq!(
            list_owner_module_lines(initializer, preamble),
            Vec::<u32>::new(),
            "{initializer}"
        );
    }
}

#[test]
fn given_shared_scope_from_a_helper_function_when_checked_then_it_is_handed_and_the_abort_is_reported()
 {
    assert_eq!(
        list_owner_module_lines("build_scope()", ""),
        [5],
        "a helper that builds the scope stays unrecognised"
    );
}

#[test]
fn given_handed_and_owned_scope_in_one_module_when_the_handed_one_is_aborted_then_it_is_reported() {
    let template = |on_disable: &str| {
        format!(
            "struct TaskScope;\nstruct CoreModule {{ task_scope: std::sync::Arc<TaskScope> }}\nmod di {{\n use super::{{CoreModule, TaskScope}};\n trait Lifecycle {{ fn on_disable(&self); }}\n struct FeatureModule {{ handed: std::sync::Arc<TaskScope>, own: TaskScope }}\n impl FeatureModule {{ fn new(core: &CoreModule) -> Self {{ Self {{ handed: std::sync::Arc::clone(&core.task_scope), own: TaskScope::new() }} }} }}\n impl Lifecycle for FeatureModule {{ fn on_disable(&self) {{ {on_disable} }} }}\n}}"
        )
    };
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        &template("self.own.abort(); self.handed.abort();"),
        &[6],
    );
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        &template("self.own.abort();"),
        &[],
    );
}

#[test]
fn given_scope_aborted_in_drop_of_its_owner_when_checked_then_the_owner_is_accepted() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new() -> Self { Self { scope: TaskScope::new() } } }\nimpl Drop for FeatureParts { fn drop(&mut self) { self.scope.abort(); } }",
        &[],
    );
}

#[test]
fn given_scope_owner_with_several_constructors_when_one_receives_the_scope_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new() -> Self { Self { scope: TaskScope::new() } } fn from_scope(scope: TaskScope) -> Self { Self { scope } } fn disable(&self) { self.scope.abort(); } }",
        &[2],
    );
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn from_defaults() -> Self { Self { scope: TaskScope::default() } } fn disable(&self) { self.scope.abort(); } }",
        &[],
    );
}

#[test]
fn given_handed_scope_stopped_through_an_alias_ufcs_or_as_ref_when_checked_then_it_is_reported() {
    for body in [
        "let scope = std::sync::Arc::clone(&self.scope); scope.abort();",
        "TaskScope::abort(&self.scope);",
        "self.scope.as_ref().abort();",
        "let scope = &self.scope; scope.reload();",
    ] {
        crate::test::module::util::assert_di_lines(
            &DiModuleLifecycle,
            &format!(
                "struct TaskScope;\nstruct FeatureSwitch {{ scope: std::sync::Arc<TaskScope> }}\nimpl FeatureSwitch {{ fn disable(&self) {{ {body} }} }}"
            ),
            &[2],
        );
    }
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureSwitch(std::sync::Arc<TaskScope>);\nimpl FeatureSwitch { fn disable(&self) { self.0.abort(); } }",
        &[2],
    );
}

#[test]
fn given_handed_scope_wrapped_in_an_option_when_stopped_then_it_is_a_recorded_gap() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureSwitch { scope: Option<std::sync::Arc<TaskScope>> }\nimpl FeatureSwitch { fn disable(&self) { if let Some(scope) = &self.scope { scope.abort(); } } }",
        &[],
    );
}

#[test]
fn given_test_only_constructor_receiving_the_scope_when_checked_then_the_scope_stays_owned() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new() -> Self { Self { scope: TaskScope::new() } } #[cfg(test)] fn with_scope(scope: TaskScope) -> Self { Self { scope } } fn disable(&self) { self.scope.abort(); } }",
        &[],
    );
}

#[test]
fn given_derived_default_on_a_type_with_a_scope_when_checked_then_the_type_builds_and_owns_it() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\n#[derive(Default)] struct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn disable(&self) { self.scope.abort(); } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n trait Lifecycle { fn on_disable(&self); }\n #[derive(Default)] struct SharedModule { task_scope: std::sync::Arc<TaskScope> }\n impl Lifecycle for SharedModule { fn on_disable(&self) { self.task_scope.abort(); } }\n}",
        &[],
    );
}

#[test]
fn given_constructors_returning_arc_or_box_of_self_when_one_receives_the_scope_then_it_is_reported()
{
    for returned in [
        "std::sync::Arc<Self>",
        "Box<Self>",
        "Result<std::sync::Arc<Self>, ()>",
    ] {
        crate::test::module::util::assert_di_lines(
            &DiModuleLifecycle,
            &format!(
                "struct TaskScope;\nstruct FeatureParts {{ scope: TaskScope }}\nimpl FeatureParts {{ fn new() -> Self {{ Self {{ scope: TaskScope::new() }} }} fn shared(scope: TaskScope) -> {returned} {{ Self {{ scope }}.into() }} fn disable(&self) {{ self.scope.abort(); }} }}"
            ),
            &[2],
        );
    }
}

#[test]
fn given_alias_shadowed_by_a_later_let_or_left_with_its_block_when_checked_then_nothing_is_reported()
 {
    for body in [
        "let scope = std::sync::Arc::clone(&self.scope); let scope = other(); scope.abort();",
        "{ let scope = &self.scope; scope.spawn(); } scope.abort();",
    ] {
        crate::test::module::util::assert_di_lines(
            &DiModuleLifecycle,
            &format!(
                "struct TaskScope;\nfn other() -> TaskScope {{ TaskScope }}\nstruct FeatureSwitch {{ scope: std::sync::Arc<TaskScope> }}\nimpl FeatureSwitch {{ fn disable(&self, scope: &TaskScope) {{ {body} }} }}"
            ),
            &[],
        );
    }
}

#[test]
fn given_non_di_type_that_builds_an_arc_scope_and_aborts_it_when_checked_then_the_message_says_to_hold_it_by_value()
 {
    let source = "struct TaskScope;\nstruct FeatureParts { scope: std::sync::Arc<TaskScope> }\nimpl FeatureParts { fn new() -> Self { Self { scope: std::sync::Arc::new(TaskScope::new()) } } fn disable(&self) { self.scope.abort(); } }";
    crate::test::module::util::assert_di_lines(&DiModuleLifecycle, source, &[2]);
    assert_eq!(
        list_lifecycle_messages(source),
        [
            "A type that is not a DI module holds the scope it builds by value, not as an `Arc<TaskScope>` (async.rs.md, Tasks)"
        ]
    );
}

#[test]
fn given_scope_owner_built_in_a_builder_a_free_function_or_a_method_chain_when_checked_then_the_conservative_reading_reports_it()
 {
    for source in [
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nfn make_parts() -> FeatureParts { FeatureParts { scope: TaskScope::new() } }\nimpl FeatureParts { fn disable(&self) { self.scope.abort(); } }",
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nstruct FeatureBuilder;\nimpl FeatureBuilder { fn build(&self) -> FeatureParts { FeatureParts { scope: TaskScope::new() } } }\nimpl FeatureParts { fn disable(&self) { self.scope.abort(); } }",
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new() -> Self { Self { scope: TaskScope::new().named(\"parts\") } } fn disable(&self) { self.scope.abort(); } }",
    ] {
        let lines: Vec<u32> = list_lifecycle_lines(source);
        assert_eq!(lines, [2], "{source}");
    }
}

fn list_lifecycle_lines(source: &str) -> Vec<u32> {
    let context = crate::test::module::util::read_di_context(source);
    DiModuleLifecycle
        .check(&context)
        .iter()
        .filter_map(|finding| finding.finding.location.line)
        .collect()
}

#[test]
fn given_task_scopes_under_exclusive_cfg_when_checked_then_they_count_once() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "#[cfg(unix)] struct TaskScope;\n#[cfg(not(unix))] struct TaskScope;",
        &[],
    );
}

#[test]
fn given_module_holding_a_shared_scope_it_was_handed_when_checked_then_abort_and_reload_are_reported()
 {
    for (parameter, argument) in [
        ("task_scope: std::sync::Arc<TaskScope>", "task_scope"),
        (
            "core: &CoreModule",
            "std::sync::Arc::clone(&core.task_scope)",
        ),
    ] {
        for call in [
            "on_disable(&self) { self.task_scope.abort(); }",
            "on_reload(&self) { self.task_scope.reload(); }",
        ] {
            crate::test::module::util::assert_di_lines(
                &DiModuleLifecycle,
                &format!(
                    "struct TaskScope;\nmod di {{\n use super::TaskScope;\n struct CoreModule {{ task_scope: std::sync::Arc<TaskScope> }}\n trait Lifecycle {{ fn on_disable(&self); fn on_reload(&self); }}\n struct FeatureModule {{ task_scope: std::sync::Arc<TaskScope> }}\n impl FeatureModule {{ fn new({parameter}) -> Self {{ Self {{ task_scope: {argument} }} }} }}\n impl Lifecycle for FeatureModule {{ fn {call} }}\n}}"
                ),
                &[6],
            );
        }
    }
}

#[test]
fn given_two_workspace_types_named_task_scope_when_checked_then_each_declaration_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod other { struct TaskScope; }",
        &[1, 2],
    );
}

#[test]
fn given_one_task_scope_declared_and_used_by_three_modules_when_checked_then_nothing_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nmod di {\n use super::TaskScope;\n struct FirstModule { task_scope: std::sync::Arc<TaskScope> }\n struct SecondModule { task_scope: std::sync::Arc<TaskScope> }\n struct ThirdModule { task_scope: std::sync::Arc<TaskScope> }\n}",
        &[],
    );
}

#[test]
fn given_a_second_task_scope_only_in_the_test_tree_when_checked_then_it_is_not_counted() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\n#[cfg(test)] mod test { struct TaskScope; }",
        &[],
    );
}

#[test]
fn given_task_scopes_of_connected_and_of_unrelated_crates_when_checked_then_only_the_connected_product_reports()
 {
    let unrelated = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_one", "", "pub struct TaskScope;"),
        ("shop_two", "", "pub struct TaskScope;"),
    ]);
    let findings = DiModuleLifecycle.check(&unrelated);
    assert!(findings.is_empty(), "{findings:?}");
    let connected = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_one", "", "pub struct TaskScope;"),
        (
            "shop_two",
            "shop_one={path=\"../shop_one\"}",
            "pub struct TaskScope;",
        ),
    ]);
    let findings = DiModuleLifecycle.check(&connected);
    let subjects: Vec<String> = findings
        .iter()
        .map(|finding| finding.finding.subject.clone())
        .collect();
    assert_eq!(subjects, ["shop_one::TaskScope", "shop_two::TaskScope"]);
    assert_eq!(
        findings[0].finding.message,
        "`TaskScope` is declared once in one product (crates linked by path dependencies); this is one of 2 declarations (async.rs.md, Tasks)"
    );
}

#[test]
fn given_a_crate_indexed_for_two_targets_when_checked_then_its_one_task_scope_is_counted_once() {
    let mut context = crate::test::module::util::read_di_context("struct TaskScope;");
    let twin =
        CrateIndexBuilder::new(&context.crates[0], context.indexes[0].dependencies.clone()).build();
    context.indexes.push(twin);
    let findings = DiModuleLifecycle.check(&context);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn given_type_that_builds_its_own_scope_and_aborts_it_when_checked_then_nothing_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new() -> Self { Self { scope: TaskScope::new() } } fn disable(&self) { self.scope.abort(); } }",
        &[],
    );
}

#[test]
fn given_type_that_builds_its_own_scope_and_never_aborts_it_when_checked_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new() -> Self { Self { scope: TaskScope::new() } } fn enable(&self) { self.scope.spawn(); } }",
        &[2],
    );
}

#[test]
fn given_type_given_a_scope_by_value_through_new_when_checked_then_it_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureParts { scope: TaskScope }\nimpl FeatureParts { fn new(scope: TaskScope) -> Self { Self { scope } } fn disable(&self) { self.scope.abort(); } }",
        &[2],
    );
}

#[test]
fn given_type_that_aborts_a_handed_scope_when_checked_then_the_handed_scope_message_is_reported() {
    let source = "struct TaskScope;\nstruct FeatureSwitch { scope: std::sync::Arc<TaskScope> }\nimpl FeatureSwitch { fn disable(&self) { self.scope.abort(); } }";
    crate::test::module::util::assert_di_lines(&DiModuleLifecycle, source, &[2]);
    assert_eq!(list_lifecycle_messages(source), [HANDED_SCOPE_MESSAGE]);
}

#[test]
fn given_type_that_reloads_a_handed_scope_when_checked_then_the_handed_scope_message_is_reported() {
    let source = "struct TaskScope;\nstruct FeatureSwitch { scope: std::sync::Arc<TaskScope> }\nimpl FeatureSwitch { fn reload(&self) { self.scope.reload(); } }";
    crate::test::module::util::assert_di_lines(&DiModuleLifecycle, source, &[2]);
    assert_eq!(list_lifecycle_messages(source), [HANDED_SCOPE_MESSAGE]);
}

#[test]
fn given_type_that_only_spawns_onto_a_handed_scope_when_checked_then_nothing_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\nstruct FeatureSwitch { scope: std::sync::Arc<TaskScope> }\nimpl FeatureSwitch { fn enable(&self) { self.scope.spawn(); } }",
        &[],
    );
}

#[test]
fn given_test_tree_fake_that_aborts_a_handed_scope_when_checked_then_nothing_is_reported() {
    crate::test::module::util::assert_di_lines(
        &DiModuleLifecycle,
        "struct TaskScope;\n#[cfg(test)] mod test {\n use super::TaskScope;\n struct FakeAbortingJob { scope: std::sync::Arc<TaskScope> }\n impl FakeAbortingJob { fn run(&self) { self.scope.abort(); } }\n}",
        &[],
    );
}

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
