use crate::check::api::Rule;
use crate::module::rule::EntryPointVisibility;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_configured_entry_kind_when_checked_then_public_types_report_and_crate_visible_types_pass()
{
    let mut context = read_shape_context(
        "mod slot { pub struct InvoiceSlot; pub(crate) struct LocalSlot; }",
        &[],
    );
    assert!(EntryPointVisibility.check(&context).is_empty());
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nentry-point-words=['slot']",
    )
    .expect("manifest");
    let findings = EntryPointVisibility.check(&context);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::slot::InvoiceSlot"
    );
}

#[test]
fn given_state_holder_interface_in_app_api_component_when_checked_then_only_the_public_command_type_reports()
 {
    let mut context = read_shape_context(
        "mod component { pub trait OfferComponent: Send + Sync { fn offer_count(&self) -> u32; } }\nmod command { pub trait InvoiceCommand {} }",
        &[],
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Api);
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nproduct-kind=\"app\"\nentry-point-words=[\"component\"]\n",
    )
    .expect("manifest");
    let findings = EntryPointVisibility.check(&context);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!("EntryPointVisibility", findings[0].finding.rule_id.0);
    assert_eq!(
        findings[0].finding.subject,
        "case_sample::command::InvoiceCommand"
    );
    assert_eq!(Some(2), findings[0].finding.location.line);
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("EntryPointVisibility", "fail-declaration-shape");
    let mut actual: Vec<String> = EntryPointVisibility
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
        ["modules/sample/src/command/invoice_command.rs:1 case_sample::command::InvoiceCommand"]
    );
    let context = read_case_context("EntryPointVisibility", "pass-declaration-shape");
    assert!(EntryPointVisibility.check(&context).is_empty());
}

fn list_findings_by_line(context: &crate::check::model::RuleContext) -> Vec<String> {
    let mut actual: Vec<String> = EntryPointVisibility
        .check(context)
        .iter()
        .map(|finding| {
            format!(
                "{} {}",
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    actual
}

#[test]
fn given_public_event_with_host_mark_when_checked_then_only_other_public_types_report() {
    let mut context = crate::test::declaration::util::read_code_context(
        "mod event {\n#[derive(vendor::Event)] pub struct PlayerJoinEvent(u32);\npub struct PlayerQuitEvent(u32);\nimpl vendor::HostEvent for PlayerQuitEvent {}\npub struct PlayerMoveEvent(u32);\n}\nmod slot { pub struct InvoiceSlot; }",
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Platform);
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nentry-point-words=['slot']",
    )
    .expect("manifest");
    assert_eq!(
        list_findings_by_line(&context),
        [
            "5 case_sample::event::PlayerMoveEvent",
            "7 case_sample::slot::InvoiceSlot"
        ]
    );
}

#[test]
fn given_platform_words_when_public_event_derives_or_implements_a_non_platform_trait_then_reports_it()
 {
    let mut context = crate::test::declaration::util::read_code_context(
        "mod event {\nuse serde::Serialize;\nuse vendor::Event;\n#[derive(Serialize)] pub struct PlayerJailedEvent(u32);\n#[derive(Event, Serialize)] pub struct PlayerJoinEvent(u32);\npub struct PlayerQuitEvent(u32);\nimpl tokio::Marker for PlayerQuitEvent {}\npub struct PlayerMoveEvent(u32);\nimpl vendor::HostEvent for PlayerMoveEvent {}\n}\nmod slot { pub struct InvoiceSlot; }",
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Platform);
    context.workspace.root_manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[workspace.metadata.klibs-rules]\nentry-point-words=['slot']",
    )
    .expect("manifest");
    context.project_config.platform_words = vec!["vendor".to_owned()];
    assert_eq!(
        list_findings_by_line(&context),
        [
            "11 case_sample::slot::InvoiceSlot",
            "4 case_sample::event::PlayerJailedEvent",
            "6 case_sample::event::PlayerQuitEvent"
        ]
    );
    context.project_config.platform_words = Vec::new();
    assert_eq!(
        list_findings_by_line(&context),
        ["11 case_sample::slot::InvoiceSlot"]
    );
}

#[test]
fn given_entry_kind_types_used_by_main_through_the_crate_name_when_checked_then_only_unused_public_types_report()
 {
    let library =
        "pub mod command {\n    pub struct InvoiceCommand;\n    pub struct OtherCommand;\n}";
    let context = crate::test::module::util::read_binary_shape_context(
        library,
        "use shop_cli::command::InvoiceCommand;\nfn main() { let command = InvoiceCommand; }",
    );
    assert_eq!(
        list_findings_by_line(&context),
        ["3 shop_cli::command::OtherCommand"]
    );
    let context = crate::test::module::util::read_binary_shape_context(library, "fn main() {}");
    assert_eq!(
        list_findings_by_line(&context),
        [
            "2 shop_cli::command::InvoiceCommand",
            "3 shop_cli::command::OtherCommand"
        ]
    );
}

#[test]
fn given_entry_kind_type_used_by_a_library_module_of_the_same_crate_when_checked_then_it_still_reports()
 {
    let context = crate::test::module::util::read_binary_shape_context(
        "pub mod command {\n    pub struct InvoiceCommand;\n}\npub mod di {\n    pub struct RootModule { command: crate::command::InvoiceCommand }\n}",
        "use shop_cli::di::RootModule;\nfn main() {}",
    );
    assert_eq!(
        list_findings_by_line(&context),
        ["2 shop_cli::command::InvoiceCommand"]
    );
}
