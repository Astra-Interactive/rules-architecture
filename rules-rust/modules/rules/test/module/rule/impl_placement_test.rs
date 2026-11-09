use crate::config::model::Role;
use crate::module::rule::ImplPlacement;
use crate::test::declaration::util::assert_code_fixture;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_inherent_extension_and_trait_impls_when_checked_then_allows_only_the_documented_files() {
    let root = "mod sample { mod model { pub(crate) use value::Value; mod value; } mod util { mod value_ext; mod wrong; } }";
    let context = read_placement_context(&[
        ("src/lib.rs", root),
        (
            "src/sample/model/value.rs",
            "pub(crate) struct Value; impl std::fmt::Display for Value {}",
        ),
        (
            "src/sample/util/value_ext.rs",
            "impl crate::sample::model::Value { fn compute(&self) {} }",
        ),
        (
            "src/sample/util/wrong.rs",
            "type Alias = crate::sample::model::Value;\nimpl Alias { fn compute(&self) {} }\nimpl std::fmt::Display for Alias {}\nimpl std::fmt::Display for tokio::Other {}",
        ),
    ]);
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[
            ("src/sample/util/wrong.rs", 2, "Node"),
            ("src/sample/util/wrong.rs", 3, "Node"),
        ],
    );
}

#[test]
fn given_model_representation_conversion_when_checked_then_accepts_the_representation_file() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod sample { mod model { pub(crate) use value::Value; mod value; } mod file { mod row; } }",
        ),
        ("src/sample/model/value.rs", "pub(crate) struct Value;"),
        (
            "src/sample/file/row.rs",
            "struct Row; impl From<Row> for crate::sample::model::Value { fn from(row: Row) -> Self { Self } }",
        ),
    ]);
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
}

#[test]
fn given_framework_error_impl_when_checked_then_requires_error_kind_of_an_entry_point() {
    let mut context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod model { pub(crate) use load_error::LoadError; mod load_error; } mod error { mod reply; }",
        ),
        (
            "src/model/load_error.rs",
            "pub(crate) enum LoadError { Missing }",
        ),
        (
            "src/error/reply.rs",
            "impl tokio::io::AsyncRead for crate::model::LoadError {}",
        ),
    ]);
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    assert_placement_findings(&ImplPlacement, "ImplPlacement", &context, &[]);
    context.project_config.crates[0].role = Some(Role::Context);
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/error/reply.rs", 1, "Node")],
    );
}

#[test]
fn given_aliased_external_conversions_when_checked_then_requires_the_matching_technology_boundary()
{
    let root = "mod model { pub(crate) use value::Value; mod value; } mod tokio { mod conversions; } mod file { mod wrong; }";
    let conversion = "use std::convert::From as Convert; type Model = crate::model::Value; type Foreign = ::tokio::Row; impl Convert<Model> for Foreign { fn from(value: Model) -> Self { loop {} } }";
    let mut context = read_placement_context(&[
        ("src/lib.rs", root),
        ("src/model/value.rs", "pub(crate) struct Value;"),
        ("src/tokio/conversions.rs", conversion),
        ("src/file/wrong.rs", conversion),
    ]);
    context.project_config.crates[0]
        .technology_words
        .push("tokio".to_owned());
    assert_placement_findings(
        &ImplPlacement,
        "ImplPlacement",
        &context,
        &[("src/file/wrong.rs", 1, "Node")],
    );
}

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&ImplPlacement, "ImplPlacement", "bad");
    assert_code_fixture(&ImplPlacement, "ImplPlacement", "good");
}
