use crate::check::api::Rule;
use crate::check::model::RuleContext;
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
fn given_host_loaded_statics_in_entry_root_when_checked_then_only_the_plain_static_reports() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    context.project_config.crates[0].host_items = vec!["PLUGIN_DECLARATION".to_owned()];
    context.crates[0].files[0].syntax = syn::parse_file(
        "pub(crate) mod di {}\npub static PLUGIN_DECLARATION: u32 = 1;\n#[unsafe(no_mangle)]\npub static PLUGIN_VERSION: u32 = 1;\nstatic COUNTER: u32 = 0;",
    )
    .expect("valid syntax");
    let lines: Vec<_> = RootFileShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(lines, [Some(5)]);
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
fn given_macro_definition_inside_a_root_module_block_when_checked_then_the_definition_reports() {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    context.crates[0].files[0].syntax = syn::parse_file(
        "pub(crate) mod invoice {\n    macro_rules! double {\n        ($value:expr) => {\n            $value * 2\n        };\n    }\n    mod record;\n}",
    )
    .expect("valid syntax");
    let findings = RootFileShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(Some(2), findings[0].finding.location.line);
    context.crates[0].files[0].syntax =
        syn::parse_file("pub(crate) mod invoice {\n    include_modules!();\n    mod record;\n}")
            .expect("valid syntax");
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

fn replace_root_syntax(context: &mut RuleContext, path: &str, source: &str) {
    let file = context
        .crates
        .iter_mut()
        .flat_map(|sources| &mut sources.files)
        .find(|file| file.path == path)
        .expect("root file of the case");
    file.syntax = syn::parse_file(source).expect("valid syntax");
}

fn list_root_lines(context: &RuleContext) -> Vec<(String, Option<u32>)> {
    let mut lines: Vec<(String, Option<u32>)> = RootFileShape
        .check(context)
        .iter()
        .map(|finding| {
            (
                finding.finding.location.path.clone(),
                finding.finding.location.line,
            )
        })
        .collect();
    lines.sort();
    lines
}

#[test]
fn given_main_of_a_crate_with_a_library_when_checked_then_only_use_lines_and_fn_main_pass() {
    let mut context = read_case_context("CrateTargets", "pass-binary-entry-point");
    assert!(RootFileShape.check(&context).is_empty());
    replace_root_syntax(
        &mut context,
        "instances/cli/src/main.rs",
        "use std::env;\nuse std::process::ExitCode;\nfn main() -> ExitCode { ExitCode::SUCCESS }",
    );
    assert!(RootFileShape.check(&context).is_empty());
}

#[test]
fn given_modules_and_other_items_in_main_of_a_crate_with_a_library_when_checked_then_each_reports_that_the_tree_is_declared_in_the_library()
 {
    let mut context = read_case_context("CrateTargets", "pass-binary-entry-point");
    replace_root_syntax(
        &mut context,
        "instances/cli/src/main.rs",
        "use std::env;\nmod di;\npub(crate) mod inline { mod record; }\nstruct Extra;\nfn helper() {}\ncall!();\npub use std::fmt::Debug;\nfn main() {}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {}\n#[cfg(any(test, feature = \"fake\"))]\n#[path = \"../fixture\"]\npub mod fixture {}",
    );
    let main = "instances/cli/src/main.rs".to_owned();
    assert_eq!(
        list_root_lines(&context),
        [
            (main.clone(), Some(2)),
            (main.clone(), Some(3)),
            (main.clone(), Some(4)),
            (main.clone(), Some(5)),
            (main.clone(), Some(6)),
            (main.clone(), Some(7)),
            (main.clone(), Some(9)),
            (main, Some(12)),
        ]
    );
    let messages: Vec<String> = RootFileShape
        .check(&context)
        .iter()
        .map(|finding| finding.finding.message.clone())
        .collect();
    assert!(
        messages
            .iter()
            .all(|message| message.contains("src/lib.rs")),
        "{messages:?}"
    );
}

#[test]
fn given_the_library_root_of_a_crate_with_a_binary_when_checked_then_it_is_judged_as_a_library_root()
 {
    let mut context = read_case_context("CrateTargets", "pass-binary-entry-point");
    let lib = "instances/cli/src/lib.rs".to_owned();
    replace_root_syntax(
        &mut context,
        &lib,
        "pub mod di { pub use root_module::RootModule; mod root_module; }\n#[cfg(any(test, feature = \"fake\"))]\n#[path = \"../fixture\"]\npub mod fixture {}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {}",
    );
    assert!(RootFileShape.check(&context).is_empty());
    replace_root_syntax(
        &mut context,
        &lib,
        "pub mod di { pub use root_module::RootModule; mod root_module; struct Extra; }\nstruct Hidden;\nfn main() {}\n#[cfg(test)]\n#[path = \"../test\"]\npub mod test {}\n#[cfg(any(test, feature = \"fake\"))]\npub mod fixture {}",
    );
    assert_eq!(
        list_root_lines(&context),
        [
            (lib.clone(), Some(1)),
            (lib.clone(), Some(2)),
            (lib.clone(), Some(3)),
            (lib.clone(), Some(4)),
            (lib.clone(), Some(7)),
            (lib, Some(7)),
        ]
    );
}

#[test]
fn given_main_alone_in_a_tooling_crate_when_checked_then_it_keeps_the_tree_as_its_root_file() {
    let mut context = read_case_context("CrateTargets", "pass-binary-entry-point");
    context.workspace.members[0]
        .targets
        .retain(|target| !target.is_lib_target());
    context
        .crates
        .retain(|sources| sources.files[0].path != "instances/cli/src/lib.rs");
    replace_root_syntax(
        &mut context,
        "instances/cli/src/main.rs",
        "use std::env;\npub(crate) mod architecture { pub(crate) mod command { mod run; } }\nfn main() {}\n#[cfg(test)]\n#[path = \"../test\"]\nmod test {}",
    );
    assert!(RootFileShape.check(&context).is_empty());
}
