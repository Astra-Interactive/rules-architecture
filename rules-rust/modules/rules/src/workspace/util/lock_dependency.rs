use crate::check::model::CheckError;
use crate::workspace::model::LockPackage;
use crate::workspace::model::LockReference;
use crate::workspace::model::TomlFile;
use crate::workspace::model::TomlValue;
use cargo_metadata::Dependency;
use cargo_metadata::Package;

pub(crate) fn build_lock_error(owner: &str, alias: &str, detail: &str) -> CheckError {
    CheckError::Configuration {
        path: "Cargo.lock".to_owned(),
        line: None,
        message: format!(
            "cannot resolve `{owner}` dependency `{alias}` to a selected package: {detail}; refresh Cargo.lock with Cargo before running architecture-check"
        ),
    }
}

fn parse_lock_package(value: &TomlValue) -> Option<LockPackage<'_>> {
    let source = match value.find(&["source"]) {
        Some(source) => Some(source.as_str()?),
        None => None,
    };
    Some(LockPackage {
        name: value.find(&["name"])?.as_str()?,
        version: value.find(&["version"])?.as_str()?,
        source,
        value,
    })
}

fn parse_lock_reference(text: &str) -> Option<LockReference<'_>> {
    let (identity, source) = match text.split_once(" (") {
        Some((identity, source)) => (identity, Some(source.strip_suffix(')')?)),
        None => (text, None),
    };
    let mut parts = identity.split_whitespace();
    let reference = LockReference {
        name: parts.next()?,
        version: parts.next(),
        source,
    };
    parts.next().is_none().then_some(reference)
}

fn matches_reference(package: &LockPackage<'_>, reference: &LockReference<'_>) -> bool {
    if package.name != reference.name {
        return false;
    }
    if reference
        .version
        .is_some_and(|version| package.version != version)
    {
        return false;
    }
    reference
        .source
        .is_none_or(|source| package.source == Some(source))
}

fn matches_requirement(dependency: &Dependency, package: &LockPackage<'_>) -> bool {
    let source_matches = package.source.is_none_or(|source| {
        dependency.source.as_ref().is_some_and(|declared| {
            source.split('#').next() == declared.to_string().split('#').next()
        })
    });
    if package.name != dependency.name || !source_matches {
        return false;
    }
    package
        .version
        .parse()
        .is_ok_and(|version| dependency.req.matches(&version))
}

fn select_reference<'lock>(
    packages: &[LockPackage<'lock>],
    reference: &LockReference<'_>,
    owner: &str,
    alias: &str,
) -> Result<usize, CheckError> {
    let mut matches = packages
        .iter()
        .enumerate()
        .filter(|(_, package)| matches_reference(package, reference));
    let selected = matches
        .next()
        .ok_or_else(|| {
            build_lock_error(
                owner,
                alias,
                "a dependency reference names no locked package",
            )
        })?
        .0;
    if matches.next().is_some() {
        if reference.version.is_some() && reference.source.is_none() {
            let local = packages
                .iter()
                .enumerate()
                .filter(|(_, package)| {
                    matches_reference(package, reference) && package.source.is_none()
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if local.len() == 1 {
                return Ok(local[0]);
            }
        }
        return Err(build_lock_error(
            owner,
            alias,
            "a dependency reference is ambiguous",
        ));
    }
    Ok(selected)
}

pub(crate) fn select_locked_dependency<'lock>(
    lock: &'lock TomlFile,
    owner: &Package,
    dependency: &Dependency,
) -> Result<LockPackage<'lock>, CheckError> {
    let alias = dependency.rename.as_deref().unwrap_or(&dependency.name);
    let error = |detail| build_lock_error(owner.name.as_str(), alias, detail);
    let values = lock
        .root
        .find(&["package"])
        .and_then(TomlValue::as_array)
        .ok_or_else(|| error("the lock contains no package table"))?;
    let mut packages: Vec<LockPackage<'_>> = values
        .iter()
        .map(|value| {
            parse_lock_package(value).ok_or_else(|| error("a locked package identity is malformed"))
        })
        .collect::<Result<_, _>>()?;
    let owner_version = owner.version.to_string();
    let mut owners = packages.iter().filter(|package| {
        let same_name = package.name == owner.name.as_str();
        let same_version = package.version == owner_version;
        let local_source = package.source.is_none();
        same_name && same_version && local_source
    });
    let locked_owner = owners
        .next()
        .ok_or_else(|| error("the owner name/version is missing or stale"))?;
    if owners.next().is_some() {
        return Err(error("the owner identity is ambiguous"));
    }
    let edges = locked_owner
        .value
        .find(&["dependencies"])
        .and_then(TomlValue::as_array)
        .ok_or_else(|| error("the owner dependency edge is missing or stale"))?;
    let mut selected = Vec::new();
    for edge in edges {
        let text = edge
            .as_str()
            .ok_or_else(|| error("a dependency reference is not a string"))?;
        let reference = parse_lock_reference(text)
            .ok_or_else(|| error("a dependency reference is malformed"))?;
        if reference.name != dependency.name {
            continue;
        }
        let index = select_reference(&packages, &reference, owner.name.as_str(), alias)?;
        if matches_requirement(dependency, &packages[index]) && !selected.contains(&index) {
            selected.push(index);
        }
    }
    if selected.len() != 1 {
        return Err(error(
            "the compatible selected dependency is missing, stale or ambiguous",
        ));
    }
    Ok(packages.swap_remove(selected[0]))
}
