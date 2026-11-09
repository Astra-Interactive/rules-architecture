use crate::workspace::util::parse_toml_file;
use crate::workspace::util::resolve_cargo_path;
use crate::workspace::util::resolve_patched_dependency;
use cargo_metadata::Metadata;
use cargo_metadata::MetadataCommand;
use std::fs;

fn read_metadata_for_dependency(declaration: &str) -> Metadata {
    let root = tempfile::tempdir().expect("temporary workspace");
    fs::write(root.path().join("Cargo.toml"), "[workspace]\nresolver=\"3\"\nmembers=[\"consumer\",\"provider\"]\n[patch.crates-io]\nlocal_provider={path=\"provider\"}\n").expect("root manifest");
    for member in ["consumer", "provider"] {
        fs::create_dir_all(root.path().join(member).join("src")).expect("member source");
    }
    fs::write(root.path().join("consumer/Cargo.toml"), format!("[package]\nname=\"consumer\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\n{declaration}\n")).expect("consumer manifest");
    fs::write(
        root.path().join("provider/Cargo.toml"),
        "[package]\nname=\"local_provider\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .expect("provider manifest");
    fs::write(root.path().join("consumer/src/lib.rs"), "").expect("consumer source");
    fs::write(root.path().join("provider/src/main.rs"), "fn main() {}").expect("provider source");
    MetadataCommand::new()
        .cargo_path(resolve_cargo_path())
        .current_dir(root.path())
        .no_deps()
        .exec()
        .expect("offline declaration metadata")
}

pub(crate) fn read_dependency_metadata() -> Metadata {
    read_metadata_for_dependency("provider_alias={package=\"local_provider\",version=\"0.1\"}")
}

#[test]
fn given_git_declaration_and_crates_io_patch_when_resolved_then_does_not_cross_source_namespaces() {
    let metadata = read_metadata_for_dependency(
        "provider_alias={package=\"local_provider\",version=\"0.1\",git=\"https://example.invalid/repository\"}",
    );
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("unrelated patch namespace");
    assert_eq!(
        None,
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            None,
            None
        )
        .expect("git remains external")
    );
}

#[test]
fn given_root_config_patches_and_named_registry_when_resolved_then_uses_effective_configuration() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"elsewhere\"}",
    )
    .expect("overridden root patch");
    let config = parse_toml_file(
        ".cargo/config.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("effective config patch");
    let lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"").expect("selected provider");
    assert_eq!(
        Some("provider".to_owned()),
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            Some(&config),
            Some(&lock)
        )
        .expect("config selection")
    );
}

#[test]
fn given_named_registry_patch_when_resolved_then_requires_its_root_config_mapping() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"").expect("selected provider");
    let named_manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.company]\nlocal_provider={path=\"provider\"}",
    )
    .expect("registry name patch");
    let named_config = parse_toml_file(
        ".cargo/config.toml",
        "[registries.company]\nindex=\"https://github.com/rust-lang/crates.io-index\"",
    )
    .expect("registry index mapping");
    assert_eq!(
        Some("provider".to_owned()),
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &named_manifest,
            Some(&named_config),
            Some(&lock)
        )
        .expect("named registry identity")
    );
    assert!(
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &named_manifest,
            None,
            Some(&lock)
        )
        .is_err()
    );
}

#[test]
fn given_config_disabling_a_manifest_patch_when_resolved_then_does_not_create_a_member_edge() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("root patch");
    let config = parse_toml_file(
        ".cargo/config.toml",
        "[patch.crates-io]\nlocal_provider={path=\"elsewhere\"}",
    )
    .expect("replacement patch");
    assert_eq!(
        None,
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            Some(&config),
            None
        )
        .expect("overridden patch is unused")
    );
}

#[test]
fn given_git_patch_namespace_when_resolved_then_keeps_selector_identity_for_external_providers() {
    let metadata = read_metadata_for_dependency(
        "provider_alias={package=\"local_provider\",version=\"0.1\",git=\"https://example.invalid/repository\",branch=\"main\"}",
    );
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.\"https://example.invalid/repository\"]\nlocal_provider={path=\"provider\"}",
    )
    .expect("git namespace patch");
    let local_lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"").expect("local git patch selection");
    assert_eq!(
        Some("provider".to_owned()),
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            None,
            Some(&local_lock)
        )
        .expect("local patch identity")
    );
    let wrong_branch_lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\nsource=\"git+https://example.invalid/repository?branch=other#hash\"").expect("different selector identity");
    assert!(
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            None,
            Some(&wrong_branch_lock)
        )
        .is_err()
    );
}

#[test]
fn given_stale_selected_local_version_or_duplicate_candidates_when_resolved_then_fails_closed() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("patch");
    let stale = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.1\"").expect("stale local identity");
    let error = resolve_patched_dependency(
        metadata.workspace_root.as_std_path(),
        owner,
        &owner.dependencies[0],
        &packages,
        &manifest,
        None,
        Some(&stale),
    )
    .expect_err("stale provider");
    assert_eq!(
        "Cargo.lock: cannot resolve `consumer` dependency `provider_alias` to a selected package: the selected local provider does not match the patched member name/version; refresh Cargo.lock with Cargo before running architecture-check",
        error.to_string()
    );
    let duplicate = parse_toml_file("Cargo.toml", "[patch.crates-io]\nfirst={package=\"local_provider\",path=\"provider\"}\nsecond={package=\"local_provider\",path=\"provider\"}").expect("duplicate local identities");
    let selected = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"").expect("selected provider");
    assert!(
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &duplicate,
            None,
            Some(&selected)
        )
        .is_err()
    );
}

#[test]
fn given_source_without_a_relevant_patch_when_resolved_then_requires_no_lock() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file("Cargo.toml", "[workspace]").expect("no patches");
    let mut dependency = owner.dependencies[0].clone();
    dependency.source = None;
    assert_eq!(
        None,
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &dependency,
            &packages,
            &manifest,
            None,
            None
        )
        .expect("source absent")
    );
}

#[test]
fn given_selected_local_patch_when_resolved_then_returns_its_member_path() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file("Cargo.toml", "[patch.crates-io]\nprovider_patch={package=\"local_provider\",path=\"provider/../provider\"}").expect("root patch");
    let lock = parse_toml_file("Cargo.lock", "version=4\n[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\n").expect("lock");
    assert_eq!(
        Some("provider".to_owned()),
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            None,
            Some(&lock)
        )
        .expect("selected local identity")
    );
}

#[test]
fn given_unused_or_incompatible_patch_when_resolved_then_keeps_the_registry_identity() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    for text in [
        "[patch.crates-io]\nother={path=\"provider\"}",
        "[patch.crates-io]\nlocal_provider={path=\"elsewhere\"}",
        "[patch.crates-io]\nlocal_provider={version=\"0.1\"}",
        "[patch.\"https://other.example/index\"]\nlocal_provider={path=\"provider\"}",
    ] {
        let manifest = parse_toml_file("Cargo.toml", text).expect("patch manifest");
        assert_eq!(
            None,
            resolve_patched_dependency(
                metadata.workspace_root.as_std_path(),
                owner,
                &owner.dependencies[0],
                &packages,
                &manifest,
                None,
                None
            )
            .expect("irrelevant patch")
        );
    }
    let mut dependency = owner.dependencies[0].clone();
    dependency.req = "^0.2".parse().expect("incompatible requirement");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("patch manifest");
    assert_eq!(
        None,
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &dependency,
            &packages,
            &manifest,
            None,
            None
        )
        .expect("incompatible patch")
    );
}

#[test]
fn given_compatible_patch_but_registry_selected_when_resolved_then_keeps_external_identity() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("root patch");
    let lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider 0.1.5 (registry+https://github.com/rust-lang/crates.io-index)\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.5\"\nsource=\"registry+https://github.com/rust-lang/crates.io-index\"\n").expect("unused patch lock");
    assert_eq!(
        None,
        resolve_patched_dependency(
            metadata.workspace_root.as_std_path(),
            owner,
            &owner.dependencies[0],
            &packages,
            &manifest,
            None,
            Some(&lock)
        )
        .expect("selected registry identity")
    );
}

#[test]
fn given_relevant_patch_without_lock_when_resolved_then_returns_actionable_exact_error() {
    let metadata = read_dependency_metadata();
    let packages = metadata.workspace_packages();
    let owner = packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let manifest = parse_toml_file(
        "Cargo.toml",
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("root patch");
    let error = resolve_patched_dependency(
        metadata.workspace_root.as_std_path(),
        owner,
        &owner.dependencies[0],
        &packages,
        &manifest,
        None,
        None,
    )
    .expect_err("missing selection evidence");
    assert_eq!(
        "Cargo.lock: cannot resolve `consumer` dependency `provider_alias` to a selected package: Cargo.lock is missing; refresh Cargo.lock with Cargo before running architecture-check",
        error.to_string()
    );
}
