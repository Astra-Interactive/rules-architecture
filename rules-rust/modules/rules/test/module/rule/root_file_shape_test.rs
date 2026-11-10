use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::RootFileShape;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_root_declarations_when_checked_then_only_ordered_platform_items_and_module_tree_pass() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[0].syntax = syn::parse_file("pub use std::fmt::Debug; mod plain; pub(crate) mod invoice { mod record; use record::Record; struct Extra; call!(); } #[cfg(test)] #[path = \"../test\"] mod test {} mod fixture {}").expect("valid syntax");
    assert_eq!(RootFileShape.check(&context).len(), 6);
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    context.project_config.crates[0].host_items = vec![
        "Plugin".to_owned(),
        "State".to_owned(),
        "host_main".to_owned(),
    ];
    context.crates[0].files[0].syntax = syn::parse_file("use std::fmt::Debug; pub(crate) mod di {} fn host_main() {} #[unsafe(no_mangle)] pub fn start() {} extern \"C\" fn stop() {} pub struct Plugin; impl Plugin { fn new() -> Self { Self } } pub enum State {} impl (u8,) {} const EXTRA: u8 = 1;").expect("valid syntax");
    assert_eq!(RootFileShape.check(&context).len(), 2);
    context.crates[0].files[0].syntax =
        syn::parse_file("#[no_mangle] pub fn start() {} fn ordinary() {}").expect("valid syntax");
    assert_eq!(RootFileShape.check(&context).len(), 1);
    context.crates[0].files[0].syntax = syn::parse_file("macro_rules! example { () => {} } pub(crate) mod invoice { use std::fmt::Debug; mod record; }").expect("valid syntax");
    assert_eq!(RootFileShape.check(&context).len(), 1);
}

#[test]
fn given_macro_definition_in_root_when_checked_then_only_the_macro_reports() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[0].syntax = syn::parse_file(
        "macro_rules! double {\n    ($value:expr) => {\n        $value * 2\n    };\n}\npub(crate) mod invoice { mod record; }",
    )
    .expect("valid syntax");
    let findings = RootFileShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("RootFileShape", findings[0].finding.rule_id.0);
    assert_eq!(Some(1), findings[0].finding.location.line);
    context.crates[0].files[0].syntax =
        syn::parse_file("pub(crate) mod invoice { mod record; }").expect("valid syntax");
    assert!(RootFileShape.check(&context).is_empty());
}

#[test]
fn given_test_block_without_cfg_test_when_checked_then_only_that_block_reports() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[0].syntax = syn::parse_file(
        "pub(crate) mod invoice { mod record; }\n#[path = \"../test\"] mod test {}",
    )
    .expect("valid syntax");
    let findings = RootFileShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("RootFileShape", findings[0].finding.rule_id.0);
    assert_eq!(Some(2), findings[0].finding.location.line);
    context.crates[0].files[0].syntax = syn::parse_file(
        "pub(crate) mod invoice { mod record; }\n#[cfg(test)] #[path = \"../test\"] mod test {}",
    )
    .expect("valid syntax");
    assert!(RootFileShape.check(&context).is_empty());
}

#[test]
fn given_public_test_block_when_checked_then_only_that_block_reports() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[0].syntax = syn::parse_file(
        "pub(crate) mod invoice { mod record; }\n#[cfg(test)] #[path = \"../test\"] pub mod test { mod order { mod usecase { mod create_order_use_case_test; } } }",
    )
    .expect("valid syntax");
    let findings = RootFileShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("RootFileShape", findings[0].finding.rule_id.0);
    assert_eq!(Some(2), findings[0].finding.location.line);
    context.crates[0].files[0].syntax = syn::parse_file(
        "pub(crate) mod invoice { mod record; }\n#[cfg(test)] #[path = \"../test\"] mod test { mod order { mod usecase { mod create_order_use_case_test; } } }",
    )
    .expect("valid syntax");
    assert!(RootFileShape.check(&context).is_empty());
}

#[test]
fn given_proc_macros_and_binary_roots_when_checked_then_required_exports_are_allowed() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.project_config.crates[0].role = Some(Role::ProcMacro);
    context.crates[0].files[0].syntax = syn::parse_file("use proc_macro::TokenStream; #[proc_macro] pub fn expand(input: TokenStream) -> TokenStream { input } #[proc_macro_derive(Model)] pub fn derive(input: TokenStream) -> TokenStream { input } #[proc_macro_attribute] pub fn attribute(input: TokenStream) -> TokenStream { input }").expect("valid syntax");
    assert!(RootFileShape.check(&context).is_empty());
    context.crates[0].files[0].syntax =
        syn::parse_file("struct Hidden; #[proc_macro] fn hidden() {} pub fn ordinary() {}")
            .expect("valid syntax");
    assert_eq!(RootFileShape.check(&context).len(), 3);
    context.project_config.crates.clear();
    context.crates[0].files[0].path = "instances/cli/src/main.rs".to_owned();
    context.workspace.members[0].targets[0].src_path = "instances/cli/src/main.rs".to_owned();
    context.workspace.members[0].targets[0].kinds = vec!["bin".to_owned()];
    context.crates[0].files[0].syntax = syn::parse_file("use std::env; pub(crate) mod di {} fn main() {} #[cfg(any(test, feature = \"fake\"))] #[path = \"../fixture\"] pub mod fixture {} #[cfg(test)] #[path = \"../test\"] mod test {}").expect("valid syntax");
    assert!(RootFileShape.check(&context).is_empty());
    context.crates[0].files[1].syntax =
        syn::parse_file("mod nested {} struct Invoice;").expect("valid syntax");
    context.crates[0]
        .undeclared_files
        .push("modules/sample/src/stray.rs".to_owned());
    assert_eq!(RootFileShape.check(&context).len(), 2);
}
