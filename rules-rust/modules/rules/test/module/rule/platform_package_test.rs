use crate::check::api::Rule;
use crate::module::rule::PlatformPackage;
use crate::test::check::usecase::read_case_context;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_target_predicates_when_checked_then_platform_module_names_are_only_reserved_in_platform_crates()
 {
    let mut context = read_case_context("PlatformPackage", "pass-concept");
    assert!(PlatformPackage.check(&context).is_empty());
    context.workspace.members[0].manifest = parse_toml_file(
        "modules/sample/Cargo.toml",
        "[target.'cfg(target_arch = \"arm\")'.dependencies]\n",
    )
    .expect("valid manifest");
    assert_eq!(PlatformPackage.check(&context).len(), 1);
    context.workspace.members[0].manifest = parse_toml_file(
        "modules/sample/Cargo.toml",
        "[target.'cfg(unix)'.dev-dependencies]\n",
    )
    .expect("valid manifest");
    assert!(PlatformPackage.check(&context).is_empty());
    context.crates[0].files[0].syntax =
        syn::parse_file("#[cfg(any(target_os = \"haiku\", unix))] struct Value;")
            .expect("valid syntax");
    context.crates[0].modules[1].id.path = vec!["haiku".to_owned()];
    assert_eq!(PlatformPackage.check(&context).len(), 1);
}
