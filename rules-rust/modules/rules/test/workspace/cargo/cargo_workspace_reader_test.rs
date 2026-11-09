use crate::check::model::CheckError;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::resolve_cargo_path;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn write_workspace(root: &Path) {
    fs::create_dir_all(root.join("modules/sample/src")).expect("the crate directory");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"modules/sample\"]\n",
    )
    .expect("the root manifest");
    fs::write(
        root.join("modules/sample/Cargo.toml"),
        "[package]\nname = \"case-sample\"\nedition = \"2024\"\n",
    )
    .expect("the member manifest");
    fs::write(root.join("modules/sample/src/lib.rs"), "\n").expect("the root file");
}

fn write_patched_workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary workspace");
    fs::write(root.path().join("Cargo.toml"), "[workspace]\nresolver=\"3\"\nmembers=[\"consumer\",\"provider\"]\n[patch.crates-io]\nlocal_provider={path=\"provider\"}\n").expect("root manifest");
    for member in ["consumer", "provider"] {
        fs::create_dir_all(root.path().join(member).join("src")).expect("member source");
    }
    fs::write(root.path().join("consumer/Cargo.toml"), "[package]\nname=\"consumer\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nprovider_alias={package=\"local_provider\",version=\"0.1\"}\n[dev-dependencies]\nprovider_alias={package=\"local_provider\",version=\"0.1\"}\n[build-dependencies]\nprovider_alias={package=\"local_provider\",version=\"0.1\"}\n[target.'cfg(unix)'.dependencies]\nprovider_alias={package=\"local_provider\",version=\"0.1\"}\n").expect("consumer manifest");
    fs::write(
        root.path().join("provider/Cargo.toml"),
        "[package]\nname=\"local_provider\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .expect("provider manifest");
    fs::write(root.path().join("consumer/src/lib.rs"), "").expect("consumer source");
    fs::write(root.path().join("provider/src/main.rs"), "fn main() {}").expect("provider source");
    root
}

#[test]
fn given_patch_in_root_cargo_config_when_read_then_resolves_the_selected_member_edge() {
    let root = write_patched_workspace();
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nresolver=\"3\"\nmembers=[\"consumer\",\"provider\"]\n",
    )
    .expect("root without patch");
    fs::create_dir(root.path().join(".cargo")).expect("root config directory");
    fs::write(
        root.path().join(".cargo/config.toml"),
        "[patch.crates-io]\nlocal_provider={path=\"provider\"}",
    )
    .expect("config patch");
    fs::write(root.path().join("Cargo.lock"), "version=4\n[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"").expect("selected lock");
    let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("root config selected patch");
    let owner = workspace
        .members
        .iter()
        .find(|member| member.name == "consumer")
        .expect("owner");
    assert!(
        owner
            .dependencies
            .iter()
            .all(|dependency| dependency.path.as_deref() == Some("provider"))
    );
}

#[test]
fn given_selected_registry_patch_to_a_member_when_read_then_preserves_each_actual_edge_kind() {
    let root = write_patched_workspace();
    let text = "version=4\n[[package]]\nname=\"consumer\"\nversion=\"0.1.0\"\ndependencies=[\"local_provider\"]\n[[package]]\nname=\"local_provider\"\nversion=\"0.1.0\"\n";
    fs::write(root.path().join("Cargo.lock"), text).expect("selected patch lock");
    let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("selected member edges");
    let owner = workspace
        .members
        .iter()
        .find(|member| member.name == "consumer")
        .expect("owner");
    assert_eq!(4, owner.dependencies.len());
    for dependency in &owner.dependencies {
        assert_eq!(Some("provider"), dependency.path.as_deref());
        assert_eq!(
            Some("local_provider"),
            dependency
                .find_member(&workspace.members)
                .map(|member| member.name.as_str())
        );
    }
    assert!(
        owner
            .dependencies
            .iter()
            .any(|dependency| dependency.kind
                == crate::workspace::model::DependencyKind::Development)
    );
    assert!(
        owner
            .dependencies
            .iter()
            .any(|dependency| dependency.kind == crate::workspace::model::DependencyKind::Build)
    );
    assert!(
        owner
            .dependencies
            .iter()
            .any(|dependency| dependency.is_target_specific)
    );
    assert_eq!(
        text,
        fs::read_to_string(root.path().join("Cargo.lock")).expect("lock remains unchanged")
    );
}

#[test]
fn given_relevant_patch_without_lock_when_read_then_fails_closed_without_creating_a_lock() {
    let root = write_patched_workspace();
    let error = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect_err("no selection evidence");
    assert_eq!(
        "Cargo.lock: cannot resolve `consumer` dependency `provider_alias` to a selected package: Cargo.lock is missing; refresh Cargo.lock with Cargo before running architecture-check",
        error.to_string()
    );
    assert!(!root.path().join("Cargo.lock").exists());
}

#[test]
fn given_readme_that_is_a_directory_when_read_then_returns_io_error() {
    let workspace = tempfile::tempdir().expect("a temporary directory");
    write_workspace(workspace.path());
    fs::create_dir_all(workspace.path().join("modules/sample/README.md"))
        .expect("a directory named README.md");

    let result = CargoWorkspaceReader::new(resolve_cargo_path()).read_workspace(workspace.path());

    assert!(
        matches!(result, Err(CheckError::Io { path, .. }) if path == "modules/sample/README.md")
    );
}

#[test]
fn given_directory_without_manifest_when_read_then_returns_metadata_error() {
    let empty = tempfile::tempdir().expect("a temporary directory");

    let result = CargoWorkspaceReader::new(resolve_cargo_path()).read_workspace(empty.path());

    assert!(matches!(result, Err(CheckError::Metadata { .. })));
}

#[cfg(unix)]
fn snapshot_cargo(root: &Path) -> PathBuf {
    let output = Command::new(resolve_cargo_path())
        .args(["metadata", "--no-deps", "--format-version=1"])
        .current_dir(root)
        .output()
        .expect("Cargo metadata");
    assert!(output.status.success());
    let snapshot = root.join("metadata.json");
    fs::write(&snapshot, output.stdout).expect("metadata snapshot");
    let command = root.join("cargo-snapshot");
    fs::write(
        &command,
        format!("#!/bin/sh\ncat '{}'\n", snapshot.display()),
    )
    .expect("command fixture");
    fs::set_permissions(&command, fs::Permissions::from_mode(0o755)).expect("executable");
    command
}

#[cfg(unix)]
#[test]
fn given_parent_segments_in_metadata_dependency_path_when_read_then_normalizes_the_member_identity()
{
    let root = write_patched_workspace();
    fs::write(root.path().join("consumer/Cargo.toml"), "[package]\nname=\"consumer\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[dependencies]\nprovider_alias={package=\"local_provider\",path=\"../provider\"}").expect("direct path manifest");
    let cargo = snapshot_cargo(root.path());
    let snapshot = root.path().join("metadata.json");
    let original = fs::read_to_string(&snapshot).expect("metadata replay");
    let canonical_root = root.path().canonicalize().expect("canonical workspace");
    let canonical_path = canonical_root.join("provider");
    let replay = original.replace(
        &format!("\"path\":\"{}\"", canonical_path.display()),
        &format!(
            "\"path\":\"{}/provider/../provider\"",
            canonical_root.display()
        ),
    );
    assert_ne!(original, replay);
    fs::write(snapshot, replay).expect("metadata with explicit parent segment");
    let workspace = CargoWorkspaceReader::new(cargo)
        .read_workspace(root.path())
        .expect("normalized direct path");
    let consumer = workspace
        .members
        .iter()
        .find(|member| member.name == "consumer")
        .expect("consumer");
    assert_eq!(Some("provider"), consumer.dependencies[0].path.as_deref());
    assert!(
        consumer.dependencies[0]
            .find_member(&workspace.members)
            .is_some()
    );
}

#[cfg(unix)]
#[test]
fn given_manifest_removed_after_metadata_when_read_then_returns_manifest_io_error() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_workspace(root.path());
    let cargo = snapshot_cargo(root.path());
    fs::remove_file(root.path().join("Cargo.toml")).expect("remove temporary manifest");
    let result = CargoWorkspaceReader::new(cargo).read_workspace(root.path());
    assert!(matches!(result, Err(CheckError::Io { path, .. }) if path == "Cargo.toml"));
}

#[cfg(unix)]
#[test]
fn given_unreadable_directory_when_walked_then_returns_its_io_error() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_workspace(root.path());
    let blocked = root.path().join("blocked");
    fs::create_dir(&blocked).expect("directory");
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).expect("deny reads");
    let result = CargoWorkspaceReader::new(resolve_cargo_path()).read_workspace(root.path());
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o755)).expect("restore reads");
    assert!(matches!(result, Err(CheckError::Io { path, .. }) if path == "blocked"));
}

#[test]
fn given_external_dependency_sharing_member_name_when_read_then_path_identity_keeps_it_external() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_workspace(root.path());
    fs::write(root.path().join("Cargo.toml"), "[workspace]\nresolver=\"3\"\nmembers=[\"modules/sample\",\"modules/log\"]\nexclude=[\"external/log\"]\n").expect("workspace manifest");
    fs::write(root.path().join("modules/sample/Cargo.toml"), "[package]\nname=\"sample\"\nedition=\"2024\"\n[dependencies]\nlogging={package=\"log\",path=\"../../external/log\"}\n").expect("renamed external dependency");
    for directory in ["modules/log", "external/log"] {
        fs::create_dir_all(root.path().join(directory).join("src")).expect("crate source");
    }
    fs::write(
        root.path().join("modules/log/Cargo.toml"),
        "[package]\nname=\"log\"\nedition=\"2024\"\n",
    )
    .expect("workspace binary");
    fs::write(root.path().join("modules/log/src/main.rs"), "fn main() {}").expect("binary source");
    fs::write(
        root.path().join("external/log/Cargo.toml"),
        "[package]\nname=\"log\"\nversion=\"0.4.0\"\nedition=\"2024\"\n",
    )
    .expect("external library");
    fs::write(root.path().join("external/log/src/lib.rs"), "").expect("library source");
    let workspace = CargoWorkspaceReader::new(resolve_cargo_path())
        .read_workspace(root.path())
        .expect("real metadata");
    let consumer = workspace
        .members
        .iter()
        .find(|member| member.name == "sample")
        .expect("consumer");
    let dependency = &consumer.dependencies[0];
    assert_eq!(dependency.name, "log");
    assert_eq!(dependency.path.as_deref(), Some("external/log"));
    assert_eq!(dependency.line, Some(5));
    assert!(dependency.find_member(&workspace.members).is_none());
}
