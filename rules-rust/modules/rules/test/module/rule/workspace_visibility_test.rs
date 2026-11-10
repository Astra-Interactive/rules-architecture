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
    assert_doc_hidden_lines(&context, &[1]);
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

#[test]
fn given_tdd_doc_hidden_private_items_when_checked_then_reports_hidden_declarations_and_members() {
    let mut context = read_code_context(
        "#[doc(hidden)] pub(crate) fn hidden_function() {}\n#[doc(hidden)] pub(crate) struct HiddenStruct {\n    #[doc(hidden)] field: u8,\n}\n#[doc(hidden)] enum HiddenEnum {\n    #[doc(hidden)] Variant,\n}\n#[doc(hidden)] trait HiddenTrait {\n    #[doc(hidden)] fn hidden_method(&self);\n}\n#[doc(hidden)] static HIDDEN_STATIC: u8 = 0;\n#[doc(hidden)] mod hidden_module {}",
    );
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published library manifest");
    let findings = DocHiddenUse.check(&context);
    assert_eq!(9, findings.len());
    assert!(
        findings
            .iter()
            .all(|finding| finding.finding.rule_id.0 == "DocHiddenUse")
    );
    let mut lines: Vec<_> = findings
        .iter()
        .map(|finding| finding.finding.location.line.expect("finding line"))
        .collect();
    lines.sort_unstable();
    assert_eq!(vec![1, 2, 3, 5, 6, 8, 9, 11, 12], lines);
}

fn assert_published_doc_hidden_lines(source: &str, expected: &[u32]) {
    let mut context = read_code_context(source);
    context.workspace.members[0].manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"case-sample\"\nversion=\"0.1.0\"",
    )
    .expect("published library manifest");
    assert_doc_hidden_lines(&context, expected);
}

fn assert_doc_hidden_lines(context: &crate::check::model::RuleContext, expected: &[u32]) {
    let findings = DocHiddenUse.check(context);
    let mut actual = findings
        .iter()
        .map(|finding| {
            assert_eq!("DocHiddenUse", finding.finding.rule_id.0);
            finding.finding.location.line.expect("finding line")
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(expected, actual, "findings: {findings:?}");
}

#[test]
fn given_doc_hidden_on_private_impl_method_when_published_then_reports_the_attribute() {
    assert_published_doc_hidden_lines(
        "pub struct Entry;\nimpl Entry {\n    #[doc(hidden)]\n    fn helper() {}\n    #[doc(hidden)]\n    pub fn bridge() {}\n    pub fn run() { Self::helper(); }\n}",
        &[3, 5],
    );
}

#[test]
fn given_conditional_doc_hidden_on_crate_private_function_when_published_then_reports_the_attribute()
 {
    assert_published_doc_hidden_lines(
        "#[cfg_attr(all(), doc(hidden))] pub(crate) fn compute(value: u32) -> u32 { value + 1 }\n#[cfg_attr(all(), doc(hidden))] pub fn bridge() {}",
        &[1, 2],
    );
}

#[test]
fn given_doc_hidden_with_alias_on_unused_public_function_when_workspace_only_then_reports_the_item()
{
    let context = read_private_context(
        "#[doc(hidden, alias = \"invoice-hint\")]\npub fn compute_hint(total: u64) -> u64 { total }\n#[doc(alias = \"invoice-total\")]\npub fn visible_total(total: u64) -> u64 { total }",
    );
    assert_doc_hidden_lines(&context, &[1]);
}

#[test]
fn given_unused_public_hidden_constant_and_method_when_workspace_only_then_reports_each() {
    let context = read_private_context(
        "pub struct Counter;\nimpl Counter {\n    #[doc(hidden)] pub fn reset_for_bridge(&self) {}\n    pub fn visible(&self) {}\n}\n#[doc(hidden)] pub const BRIDGE_LIMIT: u32 = 3;",
    );
    assert_doc_hidden_lines(&context, &[3, 6]);
}

#[test]
fn given_unused_public_hidden_function_when_published_then_reports_the_item() {
    assert_published_doc_hidden_lines(
        "#[doc(hidden)]\npub fn compute_total(subtotal: u32, tax: u32) -> u32 { subtotal + tax }\npub fn visible_total(subtotal: u32, tax: u32) -> u32 { subtotal + tax }",
        &[1],
    );
}

#[test]
fn given_hidden_public_function_named_by_another_workspace_crate_when_published_then_reports_nothing()
 {
    let mut context = crate::test::module::util::read_workspace_shape_context(&[
        (
            "shop_total_api",
            "",
            "#[doc(hidden)]\npub fn compute_total(subtotal: u32, tax: u32) -> u32 { subtotal + tax }",
        ),
        (
            "shop_total_user",
            "shop_total_api={path=\"../shop_total_api\"}",
            "fn total() -> u32 { shop_total_api::compute_total(1, 2) }",
        ),
    ]);
    let api = context
        .workspace
        .members
        .iter_mut()
        .find(|member| member.name == "shop_total_api")
        .expect("shop_total_api member");
    api.manifest = crate::workspace::util::parse_toml_file(
        "Cargo.toml",
        "[package]\nname=\"shop_total_api\"\nversion=\"0.1.0\"",
    )
    .expect("published library manifest");
    assert_doc_hidden_lines(&context, &[]);
}

#[test]
fn given_hidden_public_union_when_workspace_only_then_reports_it_unless_another_crate_names_it() {
    let context =
        read_private_context("#[doc(hidden)]\npub union Bits { pub raw: u32, pub parts: [u8; 4] }");
    assert_doc_hidden_lines(&context, &[1]);
    let context = crate::test::module::util::read_workspace_shape_context(&[
        (
            "shop_bits_api",
            "",
            "#[doc(hidden)]\npub union Bits { pub raw: u32, pub parts: [u8; 4] }",
        ),
        (
            "shop_bits_user",
            "shop_bits_api={path=\"../shop_bits_api\"}",
            "fn read(bits: shop_bits_api::Bits) -> u32 { 0 }",
        ),
    ]);
    assert_doc_hidden_lines(&context, &[]);
}

#[test]
fn given_hidden_trait_member_when_another_crate_implements_it_then_the_implementation_is_a_use() {
    let api = "pub trait Plugin {\n    #[doc(hidden)]\n    fn register(&self);\n}";
    let user =
        "pub struct Audit;\nimpl shop_plugin_api::Plugin for Audit {\n    fn register(&self) {}\n}";
    let dependency = "shop_plugin_api={path=\"../shop_plugin_api\"}";
    let context = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_plugin_api", "", api),
        ("shop_plugin_user", dependency, user),
    ]);
    assert_doc_hidden_lines(&context, &[]);
    let context = crate::test::module::util::read_workspace_shape_context(&[
        ("shop_plugin_api", "", api),
        ("shop_plugin_user", dependency, "fn unrelated() {}"),
    ]);
    assert_doc_hidden_lines(&context, &[2]);
}
