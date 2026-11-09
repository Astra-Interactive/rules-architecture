use crate::naming::rule::RolelessTypeName;
use crate::test::declaration::util::assert_code_findings;
use crate::test::declaration::util::assert_code_fixture;

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
