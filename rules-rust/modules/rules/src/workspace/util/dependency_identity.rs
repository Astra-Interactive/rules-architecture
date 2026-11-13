use crate::check::model::CheckError;
use crate::workspace::model::TomlFile;
use crate::workspace::model::TomlValue;
use crate::workspace::util::build_lock_error;
use crate::workspace::util::select_locked_dependency;
use cargo_metadata::Dependency;
use cargo_metadata::Package;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            value => normalized.push(value.as_os_str()),
        }
    }
    normalized
}

fn matches_patch_source(source: &str, patch_source: &str) -> bool {
    if patch_source == "crates-io" {
        return source == "registry+https://github.com/rust-lang/crates.io-index"
            || source == "registry+sparse+https://index.crates.io/";
    }
    let source = source
        .strip_prefix("registry+")
        .or_else(|| source.strip_prefix("git+"))
        .unwrap_or(source);
    let source = source.split(['?', '#']).next().unwrap_or(source);
    source == patch_source
}

fn find_local_patch_members<'members>(
    root: &Path,
    name: &str,
    patch: &TomlValue,
    dependency: &Dependency,
    members: &'members [&Package],
) -> Vec<&'members Package> {
    let name = patch
        .find(&["package"])
        .and_then(TomlValue::as_str)
        .unwrap_or(name);
    let Some(path) = patch.find(&["path"]).and_then(TomlValue::as_str) else {
        return Vec::new();
    };
    if name != dependency.name {
        return Vec::new();
    }
    let path = normalize_path(&root.join(path).join("Cargo.toml"));
    members
        .iter()
        .copied()
        .filter(|member| {
            if member.name.as_str() != name {
                return false;
            }
            if !dependency.req.matches(&member.version) {
                return false;
            }
            normalize_path(member.manifest_path.as_std_path()) == path
        })
        .collect()
}

fn resolve_patch_namespace<'config>(
    name: &'config str,
    config: Option<&'config TomlFile>,
) -> Option<&'config str> {
    if name == "crates-io" || name.contains("://") {
        return Some(name);
    }
    config?.root.find(&["registries", name, "index"])?.as_str()
}

fn find_patch_members<'members>(
    root: &Path,
    manifest: &TomlFile,
    config: Option<&TomlFile>,
    dependency: &Dependency,
    members: &'members [&Package],
    owner: &str,
) -> Result<Vec<&'members Package>, CheckError> {
    let Some(source) = dependency.source.as_ref().map(ToString::to_string) else {
        return Ok(Vec::new());
    };
    let mut candidates = Vec::new();
    for (index, document) in [Some(manifest), config].into_iter().flatten().enumerate() {
        let patches = document
            .root
            .find(&["patch"])
            .map(TomlValue::entries)
            .unwrap_or_default();
        for source_patch in patches {
            let namespace = resolve_patch_namespace(&source_patch.key, config);
            if namespace.is_some_and(|namespace| !matches_patch_source(&source, namespace)) {
                continue;
            }
            for patch in source_patch.value.entries() {
                let overridden = index == 0
                    && config.is_some_and(|config| {
                        config
                            .root
                            .find(&["patch", &source_patch.key, &patch.key])
                            .is_some()
                    });
                if overridden {
                    continue;
                }
                let local =
                    find_local_patch_members(root, &patch.key, &patch.value, dependency, members);
                if namespace.is_none() && !local.is_empty() {
                    let alias = dependency.rename.as_deref().unwrap_or(&dependency.name);
                    return Err(build_lock_error(
                        owner,
                        alias,
                        "a relevant patch registry name cannot be resolved from the root Cargo configuration",
                    ));
                }
                candidates.extend(local);
            }
        }
    }
    Ok(candidates)
}

pub(crate) fn resolve_patched_dependency(
    root: &Path,
    owner: &Package,
    dependency: &Dependency,
    members: &[&Package],
    manifest: &TomlFile,
    config: Option<&TomlFile>,
    lock: Option<&TomlFile>,
) -> Result<Option<String>, CheckError> {
    let candidates = find_patch_members(
        root,
        manifest,
        config,
        dependency,
        members,
        owner.name.as_str(),
    )?;
    if candidates.is_empty() {
        return Ok(None);
    }
    let alias = dependency.rename.as_deref().unwrap_or(&dependency.name);
    let error = |detail| build_lock_error(owner.name.as_str(), alias, detail);
    let lock = lock.ok_or_else(|| error("Cargo.lock is missing"))?;
    let selected = select_locked_dependency(lock, owner, dependency)?;
    if selected.source.is_some() {
        return Ok(None);
    }
    let mut matching = candidates.iter().filter(|member| {
        let same_name = member.name.as_str() == selected.name;
        let same_version = member.version.to_string() == selected.version;
        same_name && same_version
    });
    let member = matching.next().ok_or_else(|| {
        error("the selected local provider does not match the patched member name/version")
    })?;
    if matching.next().is_some() {
        return Err(error("the selected local provider path is ambiguous"));
    }
    let directory = member.manifest_path.as_std_path().with_file_name("");
    let path = directory.strip_prefix(root).unwrap_or(&directory);
    Ok(Some(
        path.components()
            .map(|part| part.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"),
    ))
}
