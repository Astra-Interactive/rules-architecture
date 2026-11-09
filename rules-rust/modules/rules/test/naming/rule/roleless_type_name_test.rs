use crate::naming::rule::RolelessTypeName;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

#[test]
fn given_factory_return_when_checked_then_requires_the_resolved_colocated_product() {
    for output in ["u64", "Vec<Product>", "other::Product", "Product", "Alias"] {
        assert_code_findings(
            &RolelessTypeName,
            "RolelessTypeName",
            &format!(
                "struct Product; type Alias = Product; mod other {{ pub struct Product; }}\ntrait ProductFactory: Send + Sync {{ fn invoke(&self) -> {output}; }}"
            ),
            if ["Product", "Alias"].contains(&output) {
                &[]
            } else {
                &[2]
            },
        );
    }
}

#[test]
fn given_cfg_products_and_dynamic_contract_when_checked_then_returns_matching_product() {
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "#[cfg(unix)] struct Product; #[cfg(not(unix))] struct Product; trait ProductFactory: Send + Sync { fn invoke(&self) -> Product; }",
        &[],
    );
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "trait Product {} trait ProductFactory: Send + Sync { fn invoke(&self) -> std::sync::Arc<dyn Product>; }",
        &[],
    );
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "struct Product; trait ProductFactory: Send + Sync { fn invoke(&self); }",
        &[1],
    );
}

#[test]
fn given_persistent_code_fixtures_when_checked_then_match_exact_anchors() {
    assert_code_fixture(&RolelessTypeName, "RolelessTypeName", "bad");
    assert_code_fixture(&RolelessTypeName, "RolelessTypeName", "good");
}

#[test]
fn given_forbidden_code_when_checked_then_reports_exact_node_lines() {
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "struct UserValidator;\nstruct UserFactory;\nstruct CacheWarmUp;\nstruct SchemaMigration;\ntrait MissingFactory: Send + Sync { fn invoke(&self); }",
        &[1, 2, 3, 4, 5],
    );
}

#[test]
fn given_prescribed_code_when_checked_then_passes() {
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "struct User;\ntrait UserFactory: Send + Sync { fn invoke(&self) -> User; }\nmod di { struct Module; impl super::UserFactory for Module { fn invoke(&self) -> super::User { super::User } } }\nmod warmup { struct CacheWarmUp; }\nmod migration { struct SchemaMigration; }",
        &[],
    );
}

#[test]
fn given_alias_factory_markers_when_checked_then_accepts_the_runtime_factory_form() {
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "use std::marker::{Send as Movable, Sync as Shared};\nstruct Product;\ntrait ProductFactory: Movable + Shared { fn invoke(&self) -> Product; }\nmod di { struct Module; impl super::ProductFactory for Module { fn invoke(&self) -> super::Product { super::Product } } }",
        &[],
    );
    assert_code_findings(
        &RolelessTypeName,
        "RolelessTypeName",
        "struct Product; trait ProductFactory: Send + Sync { fn invoke(&self) -> Product; }\nstruct Module; impl ProductFactory for Module { fn invoke(&self) -> Product { Product } }",
        &[2],
    );
}
#[test]
fn given_factory_from_another_workspace_crate_when_implemented_then_actual_di_role_is_required() {
    use crate::check::api::Rule;
    for implementor in ["Worker", "ConsumerModule"] {
        let source = format!(
            "mod di {{ struct {implementor}; impl provider::api::ProductFactory for {implementor} {{ fn invoke(&self) -> provider::api::Product {{ provider::api::Product }} }} }}"
        );
        let context = crate::test::module::util::read_workspace_shape_context(&[
            (
                "provider",
                "",
                "pub mod api { pub struct Product; pub trait ProductFactory: Send + Sync { fn invoke(&self) -> Product; } }",
            ),
            ("consumer", "provider={path=\"../provider\"}", &source),
        ]);
        assert_eq!(
            RolelessTypeName.check(&context).len(),
            usize::from(implementor == "Worker")
        );
    }
}
