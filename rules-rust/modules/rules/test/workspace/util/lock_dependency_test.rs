use crate::test::workspace::util::read_dependency_metadata;
use crate::workspace::util::parse_toml_file;
use crate::workspace::util::select_locked_dependency;

fn assert_lock_error(text: &str, detail: &str) {
    let metadata = read_dependency_metadata();
    let owner = metadata
        .packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let lock = parse_toml_file("Cargo.lock", text).expect("valid lock TOML");
    let error = select_locked_dependency(&lock, owner, &owner.dependencies[0])
        .err()
        .expect("invalid identity evidence");
    assert_eq!(
        format!(
            "Cargo.lock: cannot resolve `consumer` dependency `provider_alias` to a selected package: {detail}; refresh Cargo.lock with Cargo before running architecture-check"
        ),
        error.to_string()
    );
}

#[test]
fn given_versioned_reference_without_a_unique_local_or_external_source_when_selected_then_reports_ambiguity()
 {
    assert_lock_error(
        "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider 0.1.0\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\nsource=\"registry+https://first.invalid\"\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\nsource=\"registry+https://second.invalid\"",
        "a dependency reference is ambiguous",
    );
}

#[test]
fn given_cargo_shortened_reference_forms_when_selected_then_distinguishes_local_and_registry_identity()
 {
    let metadata = read_dependency_metadata();
    let owner = metadata
        .packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider 0.1.0\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\nsource=\"registry+https://github.com/rust-lang/crates.io-index\"").expect("same version local and registry lock");
    assert_eq!(
        None,
        select_locked_dependency(&lock, owner, &owner.dependencies[0])
            .expect("omitted local source")
            .source
    );
    let lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider 0.1.1\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.1\"\nsource=\"registry+https://github.com/rust-lang/crates.io-index\"").expect("different version unique registry reference");
    assert_eq!(
        Some("registry+https://github.com/rust-lang/crates.io-index"),
        select_locked_dependency(&lock, owner, &owner.dependencies[0])
            .expect("unique version source")
            .source
    );
}

#[test]
fn given_invalid_package_or_owner_identity_when_selected_then_returns_exact_failure() {
    assert_lock_error("version=4", "the lock contains no package table");
    assert_lock_error(
        "[[package]]\nname=\"consumer\"",
        "a locked package identity is malformed",
    );
    assert_lock_error(
        "[[package]]\nname=\"consumer\"\nversion=\"0.2.0\"",
        "the owner name/version is missing or stale",
    );
    assert_lock_error(
        "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\n[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"",
        "the owner identity is ambiguous",
    );
    assert_lock_error(
        "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\nsource=7",
        "a locked package identity is malformed",
    );
}

#[test]
fn given_malformed_or_missing_dependency_reference_when_selected_then_returns_exact_failure() {
    let owner = "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\n";
    assert_lock_error(owner, "the owner dependency edge is missing or stale");
    assert_lock_error(
        &format!("{owner}dependencies=[7]"),
        "a dependency reference is not a string",
    );
    for reference in ["", "local_provider 0.1.0 extra", "local_provider (invalid"] {
        assert_lock_error(
            &format!("{owner}dependencies=[{reference:?}]"),
            "a dependency reference is malformed",
        );
    }
    assert_lock_error(
        &format!("{owner}dependencies=[\"local_provider\"]"),
        "a dependency reference names no locked package",
    );
    assert_lock_error(
        &format!("{owner}dependencies=[\"other\"]\n[[package]]\nname=\"other\"\nversion=\"1.0.0\""),
        "the compatible selected dependency is missing, stale or ambiguous",
    );
}

#[test]
fn given_multiple_or_incompatible_selected_providers_when_selected_then_returns_exact_failure() {
    let owner = "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\n";
    let providers = "[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.1\"\n";
    assert_lock_error(
        &format!("{owner}dependencies=[\"local_provider\"]\n{providers}"),
        "a dependency reference is ambiguous",
    );
    assert_lock_error(
        &format!(
            "{owner}dependencies=[\"local_provider 0.1.0\",\"local_provider 0.1.1\"]\n{providers}"
        ),
        "the compatible selected dependency is missing, stale or ambiguous",
    );
    assert_lock_error(
        &format!(
            "{owner}dependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.2.0\""
        ),
        "the compatible selected dependency is missing, stale or ambiguous",
    );
}

#[test]
fn given_version_and_source_disambiguators_when_selected_then_uses_the_exact_provider() {
    let metadata = read_dependency_metadata();
    let owner = metadata
        .packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let lock = parse_toml_file("Cargo.lock", "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)\",\"other\",\"local_provider 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\nsource=\"registry+https://github.com/rust-lang/crates.io-index\"\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\nsource=\"git+https://example.invalid/repo#hash\"\n").expect("disambiguated lock");
    let provider = select_locked_dependency(&lock, owner, &owner.dependencies[0])
        .expect("selected registry source");
    assert_eq!(
        Some("registry+https://github.com/rust-lang/crates.io-index"),
        provider.source
    );
}

#[test]
fn given_stale_or_ambiguous_lock_when_selected_then_returns_exact_resolution_error() {
    let metadata = read_dependency_metadata();
    let owner = metadata
        .packages
        .iter()
        .find(|package| package.name == "consumer")
        .expect("owner");
    let cases = [
        "version=4",
        "[[package]]\nname=\"consumer\"\nversion=\"0.2.0\"",
        "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"",
        "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]",
        "[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.1\"",
    ];
    for text in cases {
        let lock = parse_toml_file("Cargo.lock", text).expect("valid TOML");
        let error = select_locked_dependency(&lock, owner, &owner.dependencies[0])
            .err()
            .expect("unusable lock");
        assert!(error.to_string().starts_with("Cargo.lock: cannot resolve `consumer` dependency `provider_alias` to a selected package:"));
        assert!(
            error
                .to_string()
                .ends_with("refresh Cargo.lock with Cargo before running architecture-check")
        );
    }
}
