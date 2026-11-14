use crate::declaration::rule::ContractShape;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

use crate::check::api::Rule;
use crate::config::util::parse_project_config;
use crate::source::api::SourceReader;
use crate::source::syn::CrateIndexBuilder;
use crate::source::syn::SynSourceReader;
use crate::test::declaration::util::read_code_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::model::Dependency;
use crate::workspace::model::DependencyKind;
use crate::workspace::util::resolve_cargo_path;

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&ContractShape, "ContractShape", "bad");
    assert_code_fixture(&ContractShape, "ContractShape", "good");
}

#[test]
fn given_real_workspace_alias_reexports_and_cfg_when_checked_then_resolves_contract_without_blind_spots()
 {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cases/code/ContractShape/workspace");
    let mut context = read_code_context("");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(&directory)
        .expect("workspace metadata");
    context.project_config =
        parse_project_config(&context.workspace).expect("fixture configuration");
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    for index in &context.indexes {
        assert!(index.unanalysable.is_empty(), "{:?}", index.unanalysable);
    }
    let findings = ContractShape.check(&context);
    let mut anchors = findings
        .iter()
        .map(|finding| {
            format!(
                "{}\t{}:{}\tNode",
                finding.finding.rule_id.0,
                finding.finding.location.path,
                finding.finding.location.line.expect("line anchor")
            )
        })
        .collect::<Vec<_>>();
    anchors.sort();
    let expected =
        std::fs::read_to_string(directory.join("expected.tsv")).expect("exact workspace findings");
    assert_eq!(expected.trim_end(), anchors.join("\n"));
}

#[test]
fn given_renamed_workspace_async_contract_when_checked_then_reports_its_unannotated_impl() {
    let mut context = read_code_context(
        "use renamed_provider::api::Reader as Contract;\nstruct Gateway;\nimpl Contract for Gateway { async fn load(&self) {} }",
    );
    let mut provider = read_code_context(
        "mod api { use async_trait::async_trait as async_contract; #[async_contract] trait Reader: Send + Sync { async fn load(&self); } }",
    );
    let mut member = provider.workspace.members.remove(0);
    member.name = "case-provider".to_owned();
    member.directory = "modules/provider".to_owned();
    let mut index = provider.indexes.remove(0);
    index.crate_name = member.name.clone();
    for module in &mut index.modules {
        module.module.crate_name = member.name.clone();
    }
    context.indexes[0]
        .dependencies
        .push("renamed_provider".to_owned());
    context.workspace.members[0].dependencies.push(Dependency {
        name: "renamed-provider".to_owned(),
        code_name: "renamed_provider".to_owned(),
        path: Some(
            context
                .workspace
                .root
                .join(&member.directory)
                .to_string_lossy()
                .into_owned(),
        ),
        kind: DependencyKind::Normal,
        features: Vec::new(),
        is_target_specific: false,
        line: Some(1),
    });
    context.workspace.members.push(member);
    context.indexes.push(index);
    let findings = ContractShape.check(&context);
    assert_eq!(1, findings.len(), "{findings:?}");
    assert_eq!("src/lib.rs", findings[0].finding.location.path);
    assert_eq!(Some(3), findings[0].finding.location.line);
}

#[test]
fn given_reexported_aliases_and_disjoint_cfg_when_checked_then_keeps_contract_identity() {
    assert_code_findings(
        &ContractShape,
        "ContractShape",
        "mod api { use std::marker::{Send as Movable, Sync as Shared}; trait Reader: Movable + Shared {} }\nmod other { #[cfg(feature = \"native\")] #[async_trait::async_trait] trait Reader: Send + Sync { async fn load(&self); } }\nstruct Gateway;\n#[cfg(not(feature = \"native\"))] impl other::Reader for Gateway { fn load(&self) {} }\nimpl api::Reader for Gateway {}",
        &[],
    );
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &ContractShape,
        "ContractShape",
        "mod api {\n    trait Missing {}\n    trait Generic<T>: Send + Sync {}\n    trait Other: Send + Sync + Clone {}\n    trait Async: Send + Sync { async fn load(&self); }\n}\nstruct Gateway;\nimpl api::Async for Gateway { async fn load(&self) {} }",
        &[2, 4, 5, 8],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &ContractShape,
        "ContractShape",
        "mod api {\n    trait Clock: Send + Sync { fn now(&self); }\n    #[async_trait::async_trait]\n    trait Gateway: Send + Sync { async fn load(&self); }\n}\nstruct DefaultGateway;\n#[async_trait::async_trait]\nimpl api::Gateway for DefaultGateway { async fn load(&self) {} }",
        &[],
    );
}

#[test]
fn given_api_streams_callbacks_handles_and_external_errors_when_checked_then_domain_results_and_static_boxstream_pass()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api { trait Events: Send + Sync { fn events(&self) -> futures::stream::BoxStream<'static, u64>; } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api {\ntrait Events: Send + Sync { fn events(&self) -> Vec<u64>; }\ntrait Jobs: Send + Sync { fn spawn(&self) -> tokio::task::JoinHandle<()>; }\ntrait Callback: Send + Sync { fn call(&self, action: Box<dyn Fn()>); }\ntrait Repository: Send + Sync { fn load(&self) -> Result<(), std::io::Error>; }\n}",
        &[2, 3, 4, 5],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod model { pub struct Error; pub struct JoinHandle; } mod api { trait Repo: Send + Sync { fn load(&self) -> Result<crate::model::JoinHandle, crate::model::Error>; } }",
        &[],
    );
}

#[test]
fn given_contract_method_returning_one_argument_external_result_when_checked_then_the_trait_reports()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod model { pub struct Invoice; pub struct InvoiceId; pub struct InvoiceError; }\nmod api {\ntrait InvoiceRepository: Send + Sync { fn load(&self, id: crate::model::InvoiceId) -> std::io::Result<crate::model::Invoice>; }\ntrait InvoiceReader: Send + Sync { fn read(&self, id: crate::model::InvoiceId) -> Result<crate::model::Invoice, crate::model::InvoiceError>; }\n}",
        &[3],
    );
}

#[test]
fn given_generic_di_aliases_when_checked_then_substituted_contract_types_are_used() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "type Events<'a, T> = futures::stream::BoxStream<'a, T>; mod api { trait Events: Send + Sync { fn events(&self) -> crate::Events<'static, u64>; } }",
        &[],
    );
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "type Events<'a, T> = futures::stream::BoxStream<'a, T>; mod api { trait Events: Send + Sync { fn events<'a>(&'a self) -> crate::Events<'a, u64>; } }",
        &[1],
    );
}

#[test]
fn given_di_alias_shape_when_checked_then_preserves_defined_exceptions() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api { trait Callback: Send + Sync { fn call<F>(&self, action: F) where F: Fn(); } }",
        &[1],
    );
}

#[test]
fn given_contract_returning_a_broadcast_or_mpsc_receiver_when_checked_then_only_the_watch_receiver_passes()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api { use tokio::sync::mpsc;\ntrait Feed: Send + Sync { fn subscribe(&self) -> tokio::sync::broadcast::Receiver<u64>; }\ntrait Queue: Send + Sync { fn drain(&self) -> mpsc::Receiver<u64>; }\ntrait Level: Send + Sync { fn level(&self) -> tokio::sync::watch::Receiver<u64>; }\n}",
        &[2, 3],
    );
}

#[test]
fn given_contract_returning_a_std_or_futures_channel_receiver_when_checked_then_the_trait_reports()
{
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api { use std::sync::mpsc;\ntrait Feed: Send + Sync { fn drain(&self) -> mpsc::Receiver<u64>; }\ntrait Queue: Send + Sync { fn drain(&self) -> futures::channel::mpsc::Receiver<u64>; }\ntrait Bulk: Send + Sync { fn drain(&self) -> futures::channel::mpsc::UnboundedReceiver<u64>; }\ntrait Level: Send + Sync { fn level(&self) -> tokio::sync::watch::Receiver<u64>; }\n}",
        &[2, 3, 4],
    );
}

#[test]
fn given_contract_method_returning_a_non_static_boxstream_under_any_name_when_checked_then_the_trait_reports()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api {\ntrait Publisher: Send + Sync { fn updates(&self) -> futures::stream::BoxStream<'_, u64>; }\ntrait Playback: Send + Sync { fn state(&self) -> futures::stream::BoxStream<'static, u64>; }\ntrait Level: Send + Sync { fn level(&self) -> tokio::sync::watch::Receiver<u64>; }\n}",
        &[2],
    );
}

#[test]
fn given_contract_method_taking_a_watch_or_std_sender_when_checked_then_reports_all_but_oneshot() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api { use tokio::sync::watch;\ntrait Publisher: Send + Sync { fn bind(&self, sink: watch::Sender<u8>); }\ntrait Feed: Send + Sync { fn bind(&self, sink: std::sync::mpsc::Sender<u8>); }\ntrait Queue: Send + Sync { fn bind(&self, sink: std::sync::mpsc::SyncSender<u8>); }\ntrait Ack: Send + Sync { fn confirm(&self, answer: tokio::sync::oneshot::Sender<u8>); }\ntrait Level: Send + Sync { fn level(&self) -> watch::Receiver<u8>; }\n}",
        &[2, 3, 4],
    );
}

#[test]
fn given_contract_method_taking_a_channel_sender_when_checked_then_reports_all_but_oneshot() {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api { use tokio::sync::mpsc;\ntrait Timer: Send + Sync { fn subscribe(&self, sink: tokio::sync::mpsc::Sender<u64>); }\ntrait Feed: Send + Sync { fn subscribe(&self, sink: tokio::sync::broadcast::Sender<u64>); }\ntrait Queue: Send + Sync { fn subscribe(&self, sink: mpsc::UnboundedSender<u64>); }\ntrait Ack: Send + Sync { fn confirm(&self, answer: tokio::sync::oneshot::Sender<u64>); }\ntrait Level: Send + Sync { fn level(&self) -> tokio::sync::watch::Receiver<u64>; }\n}",
        &[2, 3, 4],
    );
}

#[test]
fn given_contract_method_returning_a_boxed_pinned_or_opaque_future_when_checked_then_the_trait_reports()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api {\n#[async_trait::async_trait] trait Reader: Send + Sync { async fn load(&self) -> u32; fn count(&self) -> u32; }\ntrait Boxed: Send + Sync { fn load(&self) -> futures::future::BoxFuture<'static, u32>; }\ntrait Pinned: Send + Sync { fn load(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = u32> + Send>>; }\ntrait Opaque: Send + Sync { fn load(&self) -> impl std::future::Future<Output = u32> + Send; }\n}",
        &[3, 4, 5],
    );
}

#[test]
fn given_contract_method_returning_a_tokio_stream_wrapper_when_checked_then_boxstream_passes_under_any_name()
 {
    crate::test::module::util::assert_di_lines(
        &crate::declaration::rule::ContractShape,
        "mod api {\ntrait Bus: Send + Sync { fn events(&self) -> futures::stream::BoxStream<'static, u64>; }\ntrait Feed: Send + Sync { fn changes(&self) -> tokio_stream::wrappers::BroadcastStream<u64>; }\ntrait Queue: Send + Sync { fn items(&self) -> tokio_stream::wrappers::ReceiverStream<u64>; }\ntrait Ticker: Send + Sync { fn changes(&self) -> futures::stream::BoxStream<'static, u64>; }\n}",
        &[3, 4],
    );
}

#[test]
fn given_generic_and_where_clause_api_traits_when_checked_then_only_missing_supertraits_report() {
    assert_code_findings(
        &ContractShape,
        "ContractShape",
        "mod api {\ntrait Generic<T>: Send + Sync {}\ntrait Bounded<T> where T: Send {}\ntrait Other: Send + Sync + Clone {}\n}",
        &[3, 4],
    );
}

#[test]
fn given_maybe_send_or_sync_supertrait_when_checked_then_it_is_not_a_real_bound() {
    assert_code_findings(
        &ContractShape,
        "ContractShape",
        "mod api {\ntrait MaybeSend: ?Send + Sync {}\ntrait MaybeSync: Send + ?Sync {}\ntrait Real: Send + Sync {}\n}",
        &[2, 3],
    );
}

#[test]
fn given_static_lifetime_supertrait_when_checked_then_only_the_clone_bound_reports() {
    assert_code_findings(
        &ContractShape,
        "ContractShape",
        "mod api {\ntrait Reader: Send + Sync + 'static {}\ntrait Other: Send + Sync + Clone {}\n}",
        &[3],
    );
}
