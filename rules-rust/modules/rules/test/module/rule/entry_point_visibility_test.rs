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
