use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::RootFileShape;
use crate::test::check::usecase::read_case_context;

#[test]
fn given_root_declarations_when_checked_then_only_ordered_platform_items_and_module_tree_pass() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[0].syntax = syn::parse_file("pub use std::fmt::Debug; mod plain; pub(crate) mod invoice { mod record; use record::Record; struct Extra; call!(); } mod test {} mod fixture {}").expect("valid syntax");
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
