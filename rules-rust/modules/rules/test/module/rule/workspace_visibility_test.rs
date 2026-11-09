use crate::check::api::Rule;
use crate::module::rule::{DiModuleVisibility, DocHiddenUse};
use crate::test::declaration::util::read_code_context;

#[test]
fn given_required_fake_visibility_and_external_test_users_when_checked_then_visibility_and_ownership_count_tests_differently()
 {
    let context = read_private_context(
        "#[cfg(any(test,feature=\"fake\"))] mod fixture {mod fake {pub struct FakeRepo;}} #[cfg(test)] mod test {mod fake {pub(crate) struct FakeRepo;}}",
    );
    assert!(
        crate::module::rule::VisibilityTier
            .check(&context)
            .is_empty()
    );
    let context = crate::test::source::util::read_resolution_context();
    let (ordinal, identity) = context
        .indexes
        .iter()
        .enumerate()
        .find_map(|(ordinal, index)| {
            index
                .items
                .iter()
                .position(|item| item.name == "TestOnly")
                .map(|identity| (ordinal, identity))
        })
        .unwrap();
    assert!(
        context
            .name_index()
            .users_of(ordinal, identity)
            .any(|usage| usage.is_test)
    );
    assert!(context.has_external_name_user(ordinal, identity));
    let branch = context.indexes[ordinal]
        .items
        .iter()
        .position(|item| item.name == "BranchModel")
        .unwrap();
    assert!(!context.has_external_name_user(ordinal, branch));
    assert!(
        !crate::module::rule::VisibilityTier
            .check(&context)
            .iter()
            .any(|finding| finding.finding.subject == "TestOnly")
    );
}

fn read_private_context(source: &str) -> crate::check::model::RuleContext {
    let mut context = read_code_context(source);
    let name = context.workspace.members[0].name.clone();
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        &format!("[package]\nname={name:?}\npublish=false\n"),
    )
    .unwrap();
    context
}

#[test]
fn given_an_external_receiver_with_a_model_field_name_when_checked_then_only_item_visibility_is_proven()
 {
    let context = crate::test::source::util::read_resolution_context();
    let findings = crate::module::rule::VisibilityTier.check(&context);
    let snapshot = findings
        .iter()
        .filter(|finding| finding.finding.subject == "Snapshot")
        .collect::<Vec<_>>();
    assert_eq!(1, snapshot.len());
    assert_eq!(Some(26), snapshot[0].finding.location.line);
    assert!(!findings.iter().any(
        |finding| finding.finding.location.path == "owner/src/lib.rs"
            && finding.finding.location.line == Some(27)
    ));
}

#[test]
fn given_persistent_unpublished_workspace_cases_when_checked_then_wrong_and_correct_visibility_match_exact_anchors()
 {
    let cases: [(&dyn Rule, &str); 5] = [
        (&DocHiddenUse, "DocHiddenUse"),
        (&DiModuleVisibility, "DiModuleVisibility"),
        (&crate::module::rule::VisibilityTier, "VisibilityTier"),
        (
            &crate::module::rule::ImplementationVisibility,
            "ImplementationVisibility",
        ),
        (&crate::module::rule::ReExportShape, "ReExportShape"),
    ];
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cases/code");
    for (rule, id) in cases {
        for half in ["bad", "good"] {
            let source =
                std::fs::read_to_string(directory.join(id).join(format!("phase4-{half}.rs")))
                    .unwrap();
            let expected =
                std::fs::read_to_string(directory.join(id).join(format!("phase4-{half}.tsv")))
                    .unwrap();
            let context = read_private_context(&source);
            assert!(context.indexes[0].unanalysable.is_empty());
            let findings = rule.check(&context);
            let actual = findings
                .iter()
                .map(|finding| {
                    let crate::check::model::Anchor::Node { file, span } = &finding.anchor else {
                        panic!("node anchor expected");
                    };
                    assert_eq!(file, &finding.finding.location.path);
                    assert_eq!(Some(span.start.line), finding.finding.location.line);
                    format!(
                        "{}\t{}:{}\tNode",
                        finding.finding.rule_id.0, file, span.start.line
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(expected.trim(), actual, "{id}/{half}");
        }
    }
}

#[test]
fn given_unconsumed_public_reexport_and_item_when_checked_then_workspace_visibility_reports_exact_nodes()
 {
    let context = read_private_context(
        "mod model {\npub struct Value;\npub use Value as Exposed;\n}\npub(crate) struct Local;",
    );
    let exports = crate::module::rule::ReExportShape.check(&context);
    assert_eq!(1, exports.len());
    assert_eq!(Some(3), exports[0].finding.location.line);
    let findings = crate::module::rule::VisibilityTier.check(&context);
    let lines = findings
        .iter()
        .map(|finding| finding.finding.location.line.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(vec![2, 5], lines);
}

#[test]
fn given_storage_or_private_memory_implementation_when_checked_then_details_stay_within_crate() {
    let context = read_private_context(
        "mod api { pub trait Repo {} }\nmod memory { pub struct Cache; impl crate::api::Repo for Cache {} }\nmod table { pub struct Row; }",
    );
    let findings = crate::module::rule::ImplementationVisibility.check(&context);
    assert_eq!(
        vec![2, 3],
        findings
            .iter()
            .map(|finding| finding.finding.location.line.unwrap())
            .collect::<Vec<_>>()
    );
    let context = read_private_context(
        "mod api { pub trait Repo {} }\nmod memory { pub(crate) struct Cache; impl crate::api::Repo for Cache {} }\nmod table { pub(crate) struct Row; }",
    );
    assert!(
        crate::module::rule::ImplementationVisibility
            .check(&context)
            .is_empty()
    );
}

#[test]
fn given_macro_generated_users_when_checked_then_missing_expansion_is_uncertainty_instead_of_unused_visibility()
 {
    let context = read_private_context(
        "pub(crate) struct Value; macro_rules! use_value {()=>{ fn consume(_:crate::Value){} };} mod consumer { use_value!(); }",
    );
    assert!(context.name_index().has_blind_spots);
    assert!(
        crate::module::rule::VisibilityTier
            .check(&context)
            .is_empty()
    );
}

#[test]
fn given_an_external_user_of_the_unique_kind_reexport_when_restricted_then_the_reexport_is_reported()
 {
    let mut context = crate::test::source::util::read_resolution_context();
    assert!(
        crate::module::rule::ReExportShape
            .check(&context)
            .iter()
            .all(|finding| finding.finding.subject != "Exported")
    );
    let binding = context
        .indexes
        .iter_mut()
        .flat_map(|index| &mut index.imports)
        .find(|binding| binding.name.as_deref() == Some("Exported"))
        .unwrap();
    binding.is_externally_public = false;
    binding.is_crate_public = true;
    let expected_line = binding.span.start.line;
    let findings = crate::module::rule::ReExportShape.check(&context);
    let export = findings
        .iter()
        .find(|finding| finding.finding.subject == "Exported")
        .unwrap();
    assert_eq!("provider/src/lib.rs", export.finding.location.path);
    assert_eq!(Some(expected_line), export.finding.location.line);
}

#[test]
fn given_private_workspace_public_items_when_visibility_checked_then_exact_unused_nodes_are_reported()
 {
    let mut context = read_code_context(
        "pub mod di {\n#[doc(hidden)]\npub struct Service;\npub struct FeatureModule;\n}\n",
    );
    let name = context.workspace.members[0].name.clone();
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        &format!("[package]\nname={name:?}\npublish=false\n"),
    )
    .unwrap();
    let hidden = DocHiddenUse.check(&context);
    let modules = DiModuleVisibility.check(&context);
    assert_eq!(1, hidden.len());
    assert_eq!(Some(2), hidden[0].finding.location.line);
    assert_eq!("Service", hidden[0].finding.subject);
    assert_eq!(1, modules.len());
    assert_eq!(Some(4), modules[0].finding.location.line);
    assert_eq!("FeatureModule", modules[0].finding.subject);
}

#[test]
fn given_external_consumers_or_restricted_visibility_when_checked_then_absent_workspace_names_do_not_trigger()
 {
    let mut context = read_code_context("pub mod di { #[doc(hidden)] pub struct FeatureModule; }");
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"1.0.0\"\n",
    )
    .unwrap();
    assert!(DocHiddenUse.check(&context).is_empty());
    assert!(DiModuleVisibility.check(&context).is_empty());
    let mut context =
        read_code_context("pub mod di { #[doc(hidden)] pub(crate) struct FeatureModule; }");
    let name = context.workspace.members[0].name.clone();
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        &format!("[package]\nname={name:?}\npublish=false\n"),
    )
    .unwrap();
    assert!(DocHiddenUse.check(&context).is_empty());
    assert!(DiModuleVisibility.check(&context).is_empty());
}
