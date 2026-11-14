use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::ImplPlacement;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_inherent_extension_and_trait_impls_when_checked_then_allows_only_the_documented_files() {
    let root = "mod sample { mod model { pub(crate) use value::Value; mod value; } mod util { mod value_ext; mod wrong; } }";
    let context = read_placement_context(&[
        ("src/lib.rs", root),
        (
            "src/sample/model/value.rs",
            "pub(crate) struct Value; impl std::fmt::Display for Value {}",
        ),
        (
            "src/sample/util/value_ext.rs",
            "impl crate::sample::model::Value { fn compute(&self) {} }",
        ),
        (
            "src/sample/util/wrong.rs",
            "type Alias = crate::sample::model::Value;\nimpl Alias { fn compute(&self) {} }\nimpl std::fmt::Display for Alias {}\nimpl std::fmt::Display for tokio::Other {}",
        ),
    ]);
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[
            ("src/sample/util/wrong.rs", 2, "Node"),
            ("src/sample/util/wrong.rs", 3, "Node"),
        ],
    );
}

#[test]
fn given_model_representation_conversion_when_checked_then_accepts_the_representation_file() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod sample { mod model { pub(crate) use value::Value; mod value; } mod file { mod row; } }",
        ),
        ("src/sample/model/value.rs", "pub(crate) struct Value;"),
        (
            "src/sample/file/row.rs",
            "struct Row; impl From<Row> for crate::sample::model::Value { fn from(row: Row) -> Self { Self } }",
        ),
    ]);
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
}

#[test]
fn given_framework_error_impl_when_checked_then_requires_error_kind_of_an_entry_point() {
    let mut context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod model { pub(crate) use load_error::LoadError; mod load_error; } mod error { mod reply; }",
        ),
        (
            "src/model/load_error.rs",
            "pub(crate) enum LoadError { Missing }",
        ),
        (
            "src/error/reply.rs",
            "impl tokio::io::AsyncRead for crate::model::LoadError {}",
        ),
    ]);
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
    context.project_config.crates[0].role = Some(Role::Context);
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/error/reply.rs", 1, "Node")],
    );
}

#[test]
fn given_aliased_external_conversions_when_checked_then_requires_the_matching_technology_boundary()
{
    let root = "mod model { pub(crate) use value::Value; mod value; } mod tokio { mod conversions; } mod file { mod wrong; }";
    let conversion = "use std::convert::From as Convert; type Model = crate::model::Value; type Foreign = ::tokio::Row; impl Convert<Model> for Foreign { fn from(value: Model) -> Self { loop {} } }";
    let mut context = read_placement_context(&[
        ("src/lib.rs", root),
        ("src/model/value.rs", "pub(crate) struct Value;"),
        ("src/tokio/conversions.rs", conversion),
        ("src/file/wrong.rs", conversion),
    ]);
    context.project_config.crates[0]
        .technology_words
        .push("tokio".to_owned());
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/file/wrong.rs", 1, "Node")],
    );
}

#[test]
fn given_hand_written_into_conversion_when_checked_then_reports_it_beside_the_accepted_from() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod sample { mod model { pub(crate) use value::Value; mod value; } mod file { mod row; } }",
        ),
        ("src/sample/model/value.rs", "pub(crate) struct Value;"),
        (
            "src/sample/file/row.rs",
            "struct Row;\nimpl Into<Row> for crate::sample::model::Value { fn into(self) -> Row { Row } }\nimpl From<Row> for crate::sample::model::Value { fn from(row: Row) -> Self { Self } }",
        ),
    ]);
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/sample/file/row.rs", 2, "Node")],
    );
}

const FOREIGN_CONTRACT_IMPLEMENTATION: &str = "use crate::api::MenuScreen;\nuse futures_executor::Inventory;\nimpl MenuScreen for Inventory {}";

fn read_foreign_contract_context(
    root: &str,
    path: &str,
    source: &str,
    role: Option<Role>,
) -> crate::check::model::RuleContext {
    let mut context = read_placement_context(&[("src/lib.rs", root), (path, source)]);
    context.project_config.crates[0].role = role;
    context.project_config.crates[0]
        .technology_words
        .push("futures".to_owned());
    context
}

const FOREIGN_CONTRACT_ROOT: &str = "mod api { pub(crate) trait MenuScreen {} }";

#[test]
fn given_contract_impl_for_a_foreign_type_alone_in_its_technology_module_when_checked_then_nothing_is_reported()
 {
    let context = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod futures {{ mod inventory_menu_screen; }}"),
        "src/futures/inventory_menu_screen.rs",
        FOREIGN_CONTRACT_IMPLEMENTATION,
        None,
    );
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
}

#[test]
fn given_contract_impl_for_a_foreign_type_in_model_when_checked_then_it_is_reported() {
    let context = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod model {{ mod inventory_menu_screen; }}"),
        "src/model/inventory_menu_screen.rs",
        FOREIGN_CONTRACT_IMPLEMENTATION,
        None,
    );
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/model/inventory_menu_screen.rs", 3, "Node")],
    );
}

#[test]
fn given_contract_impl_for_a_foreign_type_in_a_file_named_otherwise_when_checked_then_it_is_reported()
 {
    let context = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod futures {{ mod screen_adapter; }}"),
        "src/futures/screen_adapter.rs",
        FOREIGN_CONTRACT_IMPLEMENTATION,
        None,
    );
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/futures/screen_adapter.rs", 3, "Node")],
    );
}

#[test]
fn given_contract_impl_for_a_foreign_type_beside_a_struct_in_one_file_when_checked_then_it_is_reported()
 {
    let context = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod futures {{ mod inventory_menu_screen; }}"),
        "src/futures/inventory_menu_screen.rs",
        &format!("{FOREIGN_CONTRACT_IMPLEMENTATION}\nstruct Wrapper;"),
        None,
    );
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/futures/inventory_menu_screen.rs", 3, "Node")],
    );
}

#[test]
fn given_contract_impl_for_a_foreign_type_directly_in_a_concept_of_a_platform_crate_when_checked_then_nothing_is_reported()
 {
    let context = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod screen {{ mod inventory_menu_screen; }}"),
        "src/screen/inventory_menu_screen.rs",
        FOREIGN_CONTRACT_IMPLEMENTATION,
        Some(Role::Platform),
    );
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
}

#[test]
fn given_ext_trait_impl_for_a_foreign_type_in_util_when_checked_then_the_new_branch_is_silent() {
    let context = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod util {{ mod inventory_ext; }}"),
        "src/util/inventory_ext.rs",
        "pub(crate) trait InventoryExt { fn compute(&self); }\nimpl InventoryExt for futures_executor::Inventory { fn compute(&self) {} }",
        None,
    );
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
}

#[test]
fn given_blanket_forwarding_impl_over_a_std_pointer_when_checked_then_the_new_branch_is_silent() {
    for self_type in [
        "std::sync::Arc<T>",
        "std::boxed::Box<T>",
        "std::rc::Rc<T>",
        "&T",
    ] {
        let context = read_foreign_contract_context(
            &format!("{FOREIGN_CONTRACT_ROOT} mod model {{ mod forwarding; }}"),
            "src/model/forwarding.rs",
            &format!(
                "use crate::api::MenuScreen;\nimpl<T: MenuScreen + ?Sized> MenuScreen for {self_type} {{}}"
            ),
            None,
        );
        assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
    }
}

#[test]
fn given_contract_impl_for_a_foreign_type_in_the_test_tree_when_checked_then_only_src_is_judged() {
    let context = read_foreign_contract_context(
        &format!(
            "{FOREIGN_CONTRACT_ROOT}\n#[cfg(test)] mod test {{ use futures_executor::Inventory; impl crate::api::MenuScreen for Inventory {{}} }}"
        ),
        "src/unused.rs",
        "",
        None,
    );
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
}

#[test]
fn given_contract_impl_for_a_std_type_when_checked_then_every_role_reports_the_wrapper_message() {
    for (role, directory) in [
        (None, "futures"),
        (Some(Role::Technology), "screen"),
        (Some(Role::Platform), "screen"),
    ] {
        let path = format!("src/{directory}/string_menu_screen.rs");
        let context = read_foreign_contract_context(
            &format!("{FOREIGN_CONTRACT_ROOT} mod {directory} {{ mod string_menu_screen; }}"),
            &path,
            "use crate::api::MenuScreen;\nimpl MenuScreen for std::string::String {}",
            role,
        );
        let findings = ImplPlacement.check(&context);
        assert_eq!(findings.len(), 1, "{role:?}: {findings:?}");
        assert_eq!(
            findings[0].finding.message,
            "A contract is not implemented for a type of the standard library: write a wrapper type (arch-package.rs.md)"
        );
    }
}

#[test]
fn given_contract_impl_for_a_prelude_std_type_when_checked_then_the_wrapper_message_is_reported() {
    for self_type in ["String", "Vec<u8>", "Option<u8>", "Box<u8>", "bool", "&str"] {
        let context = read_foreign_contract_context(
            &format!("{FOREIGN_CONTRACT_ROOT} mod futures {{ mod string_menu_screen; }}"),
            "src/futures/string_menu_screen.rs",
            &format!("use crate::api::MenuScreen;\nimpl MenuScreen for {self_type} {{}}"),
            None,
        );
        let findings = ImplPlacement.check(&context);
        assert_eq!(findings.len(), 1, "{self_type}: {findings:?}");
        assert!(
            findings[0].finding.message.contains("wrapper type"),
            "{self_type}"
        );
    }
}

#[test]
fn given_forwarding_impl_over_a_pointer_to_the_same_contract_object_when_checked_then_it_is_silent()
{
    for self_type in [
        "std::sync::Arc<dyn MenuScreen>",
        "std::rc::Rc<dyn MenuScreen>",
        "std::boxed::Box<dyn MenuScreen>",
        "&dyn MenuScreen",
        "std::sync::Arc<dyn MenuScreen + Send>",
    ] {
        let context = read_foreign_contract_context(
            &format!("{FOREIGN_CONTRACT_ROOT} mod model {{ mod forwarding; }}"),
            "src/model/forwarding.rs",
            &format!("use crate::api::MenuScreen;\nimpl MenuScreen for {self_type} {{}}"),
            None,
        );
        assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
    }
    let other = read_foreign_contract_context(
        &format!("{FOREIGN_CONTRACT_ROOT} mod model {{ mod forwarding; }}"),
        "src/model/forwarding.rs",
        "use crate::api::MenuScreen;\ntrait Other {}\nimpl MenuScreen for std::sync::Arc<dyn Other> {}",
        None,
    );
    assert_eq!(ImplPlacement.check(&other).len(), 1);
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ImplPlacement, "ImplPlacement", "bad");
    assert_code_fixture(&ImplPlacement, "ImplPlacement", "good");
}
