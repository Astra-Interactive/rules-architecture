use crate::check::api::Rule;
use crate::module::rule::EntryPointShape;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_cdylib_host_items_when_checked_then_only_hidden_configured_host_reports() {
    let mut context = read_shape_context("fn start_host() {} pub fn shutdown_host() {}", &[]);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    context.project_config.crates[0].host_items =
        vec!["start_host".to_owned(), "shutdown_host".to_owned()];
    context.workspace.members[0].targets[0].kinds = vec!["cdylib".to_owned()];
    let findings = EntryPointShape.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.subject, "case_sample::start_host");
    context.project_config.crates[0].host_items.clear();
    assert!(EntryPointShape.check(&context).is_empty());
}

fn read_staticlib_host_context(source: &str) -> crate::check::model::RuleContext {
    let mut context = read_shape_context(source, &[]);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    context.project_config.crates[0].host_items = vec!["start_host".to_owned()];
    context.workspace.members[0].targets[0].kinds = vec!["staticlib".to_owned()];
    context
}

#[test]
fn given_staticlib_host_items_when_checked_then_only_hidden_configured_host_reports() {
    let context =
        read_staticlib_host_context("#[unsafe(no_mangle)] extern \"C\" fn start_host() {}");
    let findings = EntryPointShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.rule_id.0, "EntryPointShape");
    assert_eq!(findings[0].finding.subject, "case_sample::start_host");
    assert_eq!(findings[0].finding.location.line, Some(1));
    let context =
        read_staticlib_host_context("#[unsafe(no_mangle)] pub extern \"C\" fn start_host() {}");
    assert!(EntryPointShape.check(&context).is_empty());
}

fn read_unconfigured_cdylib_context(source: &str) -> crate::check::model::RuleContext {
    let mut context = read_shape_context(source, &[]);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    context.project_config.crates[0].host_items.clear();
    context.workspace.members[0].targets[0].kinds = vec!["cdylib".to_owned()];
    context
}

#[test]
fn given_cdylib_without_host_items_when_host_function_is_private_then_only_it_reports() {
    let context =
        read_unconfigured_cdylib_context("#[unsafe(no_mangle)] extern \"C\" fn plugin_start() {}");
    let findings = EntryPointShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.rule_id.0, "EntryPointShape");
    assert_eq!(findings[0].finding.subject, "case_sample::plugin_start");
    assert_eq!(findings[0].finding.location.line, Some(1));
    let context = read_unconfigured_cdylib_context(
        "#[unsafe(no_mangle)] pub extern \"C\" fn plugin_start() {}",
    );
    assert!(EntryPointShape.check(&context).is_empty());
}

#[test]
fn given_cdylib_host_static_when_checked_then_only_the_private_one_reports() {
    let context = read_unconfigured_cdylib_context(
        "#[unsafe(no_mangle)] pub static PLUGIN_VERSION: u32 = 1;",
    );
    assert!(EntryPointShape.check(&context).is_empty());
    let context =
        read_unconfigured_cdylib_context("#[unsafe(no_mangle)] static PLUGIN_VERSION: u32 = 1;");
    let findings = EntryPointShape.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].finding.subject, "case_sample::PLUGIN_VERSION");
    assert_eq!(findings[0].finding.location.line, Some(1));
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("EntryPointShape", "fail-declaration-shape");
    let mut actual: Vec<String> = EntryPointShape
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        [
            "instances/runner/src/main.rs:1 case_runner::model",
            "instances/runner/src/main.rs:2 case_runner::main"
        ]
    );
    let context = read_case_context("EntryPointShape", "pass-declaration-shape");
    assert!(EntryPointShape.check(&context).is_empty());
}

fn read_binary_main_context(source: &str) -> crate::check::model::RuleContext {
    let mut context = read_shape_context(source, &[]);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    context.workspace.members[0].targets[0].kinds = vec!["bin".to_owned()];
    context
}

#[test]
fn given_restricted_visibility_on_binary_main_when_checked_then_each_spelling_reports() {
    for visibility in ["pub(crate)", "pub(super)", "pub(in crate)"] {
        let context = read_binary_main_context(&format!("{visibility} fn main() {{}}"));
        let findings = EntryPointShape.check(&context);
        assert_eq!(findings.len(), 1, "{visibility}: {findings:?}");
        assert_eq!(findings[0].finding.rule_id.0, "EntryPointShape");
        assert_eq!(findings[0].finding.subject, "case_sample::main");
        assert_eq!(findings[0].finding.location.line, Some(1));
    }
    let context = read_binary_main_context("fn main() {}");
    assert!(EntryPointShape.check(&context).is_empty());
}

#[test]
fn given_extern_abi_functions_in_cdylib_when_checked_then_only_exported_symbols_report() {
    let context = read_unconfigured_cdylib_context(
        "extern \"C\" fn trampoline(signal: i32) {}\n#[unsafe(no_mangle)] extern \"C\" fn plugin_start() {}\n#[export_name = \"plugin_stop\"] extern \"C\" fn plugin_stop() {}\n#[unsafe(no_mangle)] pub extern \"C\" fn plugin_flush() {}",
    );
    let findings = EntryPointShape.check(&context);
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.rule_id.0 == "EntryPointShape")
    );
    let mut lines: Vec<u32> = findings
        .iter()
        .filter_map(|finding| finding.finding.location.line)
        .collect();
    lines.sort_unstable();
    assert_eq!(vec![2, 3], lines, "{findings:?}");
}
