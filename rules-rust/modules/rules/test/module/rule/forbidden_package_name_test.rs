use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::config::model::Role;
use crate::module::rule::ForbiddenPackageName;
use crate::test::module::util::assert_import_pair;
use crate::test::module::util::read_import_context;

#[test]
fn given_persistent_phase3_package_pairs_when_checked_then_only_the_technology_role_allows_storage_kinds()
 {
    assert_import_pair(
        &ForbiddenPackageName,
        "ForbiddenPackageName",
        Role::Technology,
    );
}

#[test]
fn given_technology_only_package_names_when_checked_then_role_controls_all_three_names_and_module_anchors()
 {
    for role in [Role::Technology, Role::Platform, Role::CorePlatform] {
        let context = read_import_context(
            "mod table {} mod entity {} mod database {}",
            role,
            &[],
            &[],
            "app",
        );
        assert!(ForbiddenPackageName.check(&context).is_empty(), "{role:?}");
    }
    for role in [
        Role::Context,
        Role::Api,
        Role::Core,
        Role::CoreApi,
        Role::Contract,
    ] {
        let context = read_import_context(
            "mod table {}\nmod entity {}\nmod database {}",
            role,
            &[],
            &[],
            "app",
        );
        let findings = ForbiddenPackageName.check(&context);
        assert_eq!(findings.len(), 3, "{role:?}");
        assert!(findings.iter().all(|finding| matches!(&finding.anchor, Anchor::Module { file, body_file, .. } if file == "src/lib.rs" && body_file == "src/lib.rs")));
    }
}
