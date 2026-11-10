use crate::check::api::Rule;
use crate::config::util::parse_project_config;
use crate::naming::rule::ImplementationName;
use crate::source::api::SourceReader;
use crate::source::syn::{CrateIndexBuilder, SynSourceReader};
use crate::test::check::usecase::read_case_context;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::declaration::util::read_code_context;
use crate::test::module::util::read_workspace_shape_context;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::resolve_cargo_path;

#[test]
fn given_documented_bridge_fakes_and_di_names_when_checked_then_keeps_their_prescribed_names() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cases/doc");
    let mut context = read_code_context("");
    context.workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(&directory)
        .expect("document fixture workspace");
    context.project_config =
        parse_project_config(&context.workspace).expect("document fixture configuration");
    context.crates = SynSourceReader.read_sources(&context.workspace);
    context.indexes = CrateIndexBuilder::build_all(&context.crates, &context.workspace);
    let endings = [
        "acme_playback_bridge.rs",
        "fake_user_repository.rs",
        "core_module.rs",
    ];
    for ending in endings {
        assert!(
            context
                .crates
                .iter()
                .flat_map(|sources| &sources.files)
                .any(|file| file.path.ends_with(ending)),
            "the prescribed {ending} input exists"
        );
        assert!(!context.indexes.iter().flat_map(|index| &index.unanalysable).any(|source| matches!(&source.site, crate::source::model::UnanalysableSite::Node { file, .. } if file.ends_with(ending))), "the prescribed {ending} resolves");
    }
    let findings = ImplementationName.check(&context);
    assert!(
        !findings
            .iter()
            .any(|finding| endings.iter().any(|ending| finding
                .finding
                .location
                .path
                .ends_with(ending))),
        "{findings:?}"
    );
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&ImplementationName, "ImplementationName", "bad");
    assert_code_fixture(&ImplementationName, "ImplementationName", "good");
}

#[test]
fn given_resolved_api_trait_alias_when_checked_then_reports_only_forbidden_technology_prefixes() {
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { trait Reader {} pub use Reader as ReadContract; }\nmod model {\n    struct Reader;\n    impl crate::api::ReadContract for Reader {}\n    struct CustomReader;\n    impl crate::api::ReadContract for CustomReader {}\n}\nmod file {\n    struct DatabaseReader;\n    impl crate::api::ReadContract for DatabaseReader {}\n}",
        &[9],
    );
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { trait Reader {} pub use Reader as ReadContract; }\nmod model { struct DefaultReader; impl crate::api::ReadContract for DefaultReader {} }\nmod file { struct FileReader; impl crate::api::ReadContract for FileReader {} }",
        &[],
    );
}

#[test]
fn given_cfg_disjoint_same_named_traits_when_checked_then_preserves_trait_identity() {
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { #[cfg(feature = \"enabled\")] trait Reader {} }\nmod file { struct DatabaseReader; #[cfg(not(feature = \"enabled\"))] impl crate::api::Reader for DatabaseReader {} }",
        &[],
    );
}

#[test]
fn given_type_alias_self_when_checked_then_names_the_underlying_implementation() {
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { trait Reader {} }\nmod model { struct DefaultReader; type Alias = DefaultReader; impl crate::api::Reader for Alias {} }",
        &[],
    );
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { trait Reader {} }\nmod model { struct NetworkReader; type Alias = NetworkReader; impl crate::api::Reader for Alias {} }",
        &[2],
    );
}

#[test]
fn given_contract_of_another_workspace_crate_when_checked_then_reports_only_vague_prefix() {
    let context = read_workspace_shape_context(&[
        (
            "shop_billing_api",
            "",
            "pub mod invoice { pub mod api { pub trait Reader {} } }",
        ),
        (
            "shop_billing_impl",
            "shop_billing_api={path=\"../shop_billing_api\"}",
            "mod file {\n    pub struct FileReader;\n    impl shop_billing_api::invoice::api::Reader for FileReader {}\n    pub struct NetworkReader;\n    impl shop_billing_api::invoice::api::Reader for NetworkReader {}\n}",
        ),
    ]);
    let findings = ImplementationName.check(&context);
    let reported: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!("ImplementationName", finding.finding.rule_id.0);
            finding.finding.location.line
        })
        .collect();
    assert_eq!(vec![Some(4)], reported, "{findings:?}");
}

#[test]
fn given_locale_prefixed_implementations_when_checked_then_reports_only_the_local_prefix() {
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { trait MessageRepository {} }\nmod file {\n    struct LocalizedMessageRepository;\n    impl crate::api::MessageRepository for LocalizedMessageRepository {}\n    struct LocaleCatalogRepository;\n    impl crate::api::MessageRepository for LocaleCatalogRepository {}\n    struct LocalMessageRepository;\n    impl crate::api::MessageRepository for LocalMessageRepository {}\n}",
        &[7],
    );
}

#[test]
fn given_interface_prefixed_struct_when_checked_then_reports_only_that_struct() {
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "mod api { trait Invoice {} }\nmod model {\n    struct IInvoice;\n    impl crate::api::Invoice for IInvoice {}\n    struct DefaultInvoice;\n    impl crate::api::Invoice for DefaultInvoice {}\n}",
        &[3],
    );
}

#[test]
fn given_type_kinds_when_checked_then_reports_only_forbidden_names() {
    crate::test::syntax::util::assert_syntax_findings(
        &ImplementationName,
        "ImplementationName",
        "struct RowImpl; enum RowInterface {} union RowImplUnion { value: u8 } trait IReader {} trait IO {} trait Invoice {} union ValueImpl { value: u8 }",
        4,
    );
}

#[test]
fn given_type_aliases_with_impl_or_interface_suffix_when_checked_then_reports_only_those() {
    assert_code_findings(
        &ImplementationName,
        "ImplementationName",
        "pub(crate) struct Jail { pub(crate) name: String }\npub(crate) type JailImpl = Jail;\npub(crate) type JailInterface = Jail;\npub(crate) type JailIndex = Jail;\ntrait Source { type Item; }\nstruct Reader; impl Source for Reader { type Item = JailImpl; }",
        &[2, 3],
    );
}

#[test]
fn given_forbidden_syntax_when_checked_then_reports_the_rule() {
    let context = read_case_context("ImplementationName", "fail-syntax");
    assert!(!ImplementationName.check(&context).is_empty());
}

#[test]
fn given_prescribed_syntax_when_checked_then_reports_nothing() {
    let context = read_case_context("ImplementationName", "pass-syntax");
    assert!(ImplementationName.check(&context).is_empty());
}
