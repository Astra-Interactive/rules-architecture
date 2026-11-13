use crate::check::model::CheckError;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::model::Dependency;
use crate::workspace::model::DependencyKind;
use crate::workspace::model::Member;
use crate::workspace::model::Target;
use crate::workspace::model::TomlFile;
use crate::workspace::model::TomlValue;
use crate::workspace::model::ToolchainFile;
use crate::workspace::model::Workspace;
use crate::workspace::util::is_member_source_directory;
use crate::workspace::util::is_skipped_directory_name;
use crate::workspace::util::parse_toml_file;
use crate::workspace::util::resolve_patched_dependency;
use cargo_metadata::Dependency as MetadataDependency;
use cargo_metadata::DependencyKind as MetadataDependencyKind;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct CargoWorkspaceReader {
    cargo: PathBuf,
}

impl CargoWorkspaceReader {
    pub(crate) fn new(cargo: PathBuf) -> Self {
        Self { cargo }
    }

    fn to_relative(root: &Path, path: &Path) -> String {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            if component == Component::ParentDir {
                normalized.pop();
            } else {
                normalized.push(component.as_os_str());
            }
        }
        let relative = normalized.strip_prefix(root).unwrap_or(&normalized);
        relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("/")
    }

    fn join_relative(directory: &str, name: &str) -> String {
        if directory.is_empty() {
            return name.to_owned();
        }
        format!("{directory}/{name}")
    }

    fn read_text(root: &Path, relative: &str) -> Result<Option<String>, CheckError> {
        match fs::read_to_string(root.join(relative)) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(CheckError::Io {
                path: relative.to_owned(),
                source: error.into(),
            }),
        }
    }

    fn read_toml(root: &Path, relative: &str) -> Result<Option<TomlFile>, CheckError> {
        let Some(text) = Self::read_text(root, relative)? else {
            return Ok(None);
        };
        parse_toml_file(relative, &text).map(Some)
    }

    fn read_manifest(root: &Path, relative: &str) -> Result<TomlFile, CheckError> {
        let text = fs::read_to_string(root.join(relative)).map_err(|error| CheckError::Io {
            path: relative.to_owned(),
            source: error.into(),
        })?;
        parse_toml_file(relative, &text)
    }

    fn read_targets(root: &Path, package: &Package) -> Vec<Target> {
        package
            .targets
            .iter()
            .map(|target| Target {
                name: target.name.clone(),
                kinds: target.kind.iter().map(ToString::to_string).collect(),
                src_path: Self::to_relative(root, target.src_path.as_std_path()),
            })
            .collect()
    }

    fn select_dependency_table(kind: DependencyKind) -> &'static str {
        match kind {
            DependencyKind::Normal => "dependencies",
            DependencyKind::Development => "dev-dependencies",
            DependencyKind::Build => "build-dependencies",
        }
    }

    fn find_dependency_line(
        manifest: &TomlFile,
        key: &str,
        kind: DependencyKind,
        target: Option<&str>,
    ) -> Option<u32> {
        let table = Self::select_dependency_table(kind);
        let Some(target) = target else {
            return manifest.root.find(&[table, key]).map(|value| value.line);
        };
        manifest
            .root
            .find(&["target", target, table, key])
            .map(|value| value.line)
    }

    fn expand_dependency_features(
        root: &Path,
        dependency: &MetadataDependency,
        path: Option<&str>,
        packages: &[&Package],
    ) -> Vec<String> {
        let Some(provider) = packages.iter().find(|provider| {
            provider.manifest_path.parent().is_some_and(|directory| {
                path == Some(Self::to_relative(root, directory.as_std_path()).as_str())
            })
        }) else {
            return dependency.features.clone();
        };
        let mut pending = dependency.features.clone();
        if dependency.uses_default_features {
            pending.push("default".to_owned());
        }
        let mut enabled = BTreeSet::new();
        while let Some(feature) = pending.pop() {
            if !enabled.insert(feature.clone()) {
                continue;
            }
            if let Some(children) = provider.features.get(&feature) {
                pending.extend(
                    children
                        .iter()
                        .filter(|child| !child.contains('/') && !child.starts_with("dep:"))
                        .cloned(),
                );
            }
        }
        enabled.into_iter().collect()
    }

    fn is_fake_of(dependency: &str, provider_feature: &str, key: &str) -> bool {
        dependency.trim_end_matches('?') == key && provider_feature == "fake"
    }

    fn is_fake_feature(child: &str, key: &str) -> bool {
        child
            .split_once('/')
            .is_some_and(|(dependency, provider_feature)| {
                Self::is_fake_of(dependency, provider_feature, key)
            })
    }

    fn list_features_enabled_by_dependents(
        package: &Package,
        packages: &[&Package],
    ) -> Vec<String> {
        packages
            .iter()
            .flat_map(|dependent| &dependent.dependencies)
            .filter(|dependency| {
                dependency.name == package.name.as_str()
                    && matches!(
                        dependency.kind,
                        MetadataDependencyKind::Normal | MetadataDependencyKind::Unknown
                    )
            })
            .flat_map(|dependency| dependency.features.iter().cloned())
            .collect()
    }

    fn enables_fake_through_features(package: &Package, key: &str, packages: &[&Package]) -> bool {
        let mut pending = Self::list_features_enabled_by_dependents(package, packages);
        pending.push("default".to_owned());
        let mut reached = BTreeSet::new();
        while let Some(feature) = pending.pop() {
            if !reached.insert(feature.clone()) {
                continue;
            }
            let Some(children) = package.features.get(&feature) else {
                continue;
            };
            if children
                .iter()
                .any(|child| Self::is_fake_feature(child, key))
            {
                return true;
            }
            pending.extend(
                children
                    .iter()
                    .filter(|child| !child.contains('/') && !child.starts_with("dep:"))
                    .cloned(),
            );
        }
        false
    }

    fn read_dependencies(
        root: &Path,
        package: &Package,
        manifest: &TomlFile,
        packages: &[&Package],
        root_manifest: &TomlFile,
        lock: Option<&TomlFile>,
        config: Option<&TomlFile>,
    ) -> Result<Vec<Dependency>, CheckError> {
        package
            .dependencies
            .iter()
            .map(|dependency| {
                let kind = match dependency.kind {
                    MetadataDependencyKind::Development => DependencyKind::Development,
                    MetadataDependencyKind::Build => DependencyKind::Build,
                    MetadataDependencyKind::Normal | MetadataDependencyKind::Unknown => {
                        DependencyKind::Normal
                    }
                };
                let is_target_specific = dependency.target.is_some();
                let target = dependency.target.as_ref().map(ToString::to_string);
                let key = dependency.rename.as_deref().unwrap_or(&dependency.name);
                let path = match &dependency.path {
                    Some(path) => Some(Self::to_relative(root, path.as_std_path())),
                    None => resolve_patched_dependency(
                        root,
                        package,
                        dependency,
                        packages,
                        root_manifest,
                        config,
                        lock,
                    )?,
                };
                let mut features =
                    Self::expand_dependency_features(root, dependency, path.as_deref(), packages);
                let enables_fake = kind != DependencyKind::Development
                    && Self::enables_fake_through_features(package, key, packages);
                if enables_fake && !features.iter().any(|feature| feature == "fake") {
                    features.push("fake".to_owned());
                }
                Ok(Dependency {
                    name: dependency.name.clone(),
                    code_name: key.replace('-', "_"),
                    path,
                    kind,
                    features,
                    is_target_specific,
                    line: Self::find_dependency_line(manifest, key, kind, target.as_deref()),
                })
            })
            .collect()
    }

    fn read_member(
        root: &Path,
        package: &Package,
        packages: &[&Package],
        root_manifest: &TomlFile,
        lock: Option<&TomlFile>,
        config: Option<&TomlFile>,
    ) -> Result<Member, CheckError> {
        let manifest_path = Self::to_relative(root, package.manifest_path.as_std_path());
        let directory = manifest_path
            .strip_suffix("Cargo.toml")
            .unwrap_or(&manifest_path)
            .trim_end_matches('/')
            .to_owned();
        let manifest = Self::read_manifest(root, &manifest_path)?;
        let readme = Self::read_text(root, &Self::join_relative(&directory, "README.md"))?;
        Ok(Member {
            name: package.name.to_string(),
            directory,
            targets: Self::read_targets(root, package),
            dependencies: Self::read_dependencies(
                root,
                package,
                &manifest,
                packages,
                root_manifest,
                lock,
                config,
            )?,
            manifest,
            readme,
        })
    }

    fn read_clippy_files(root: &Path, members: &[Member]) -> Result<Vec<TomlFile>, CheckError> {
        let mut directories = vec![String::new()];
        directories.extend(
            members
                .iter()
                .map(|member| member.directory.clone())
                .filter(|directory| !directory.is_empty()),
        );
        let mut files = Vec::new();
        for directory in directories {
            for name in ["clippy.toml", ".clippy.toml"] {
                if let Some(file) = Self::read_toml(root, &Self::join_relative(&directory, name))? {
                    files.push(file);
                }
            }
        }
        Ok(files)
    }

    fn read_rustfmt_files(root: &Path, members: &[Member]) -> Result<Vec<TomlFile>, CheckError> {
        let mut directories = vec![String::new()];
        directories.extend(
            members
                .iter()
                .map(|member| member.directory.clone())
                .filter(|directory| !directory.is_empty()),
        );
        let mut files = Vec::new();
        for directory in directories {
            for name in ["rustfmt.toml", ".rustfmt.toml"] {
                if let Some(file) = Self::read_toml(root, &Self::join_relative(&directory, name))? {
                    files.push(file);
                }
            }
        }
        Ok(files)
    }

    fn read_toolchain_file(root: &Path) -> Result<Option<ToolchainFile>, CheckError> {
        if let Some(file) = Self::read_toml(root, "rust-toolchain.toml")? {
            return Ok(Some(ToolchainFile::Toml(file)));
        }
        let Some(text) = Self::read_text(root, "rust-toolchain")? else {
            return Ok(None);
        };
        if text.contains("[toolchain]") {
            return parse_toml_file("rust-toolchain", &text)
                .map(|file| Some(ToolchainFile::Toml(file)));
        }
        Ok(Some(ToolchainFile::Legacy {
            path: "rust-toolchain".to_owned(),
            channel: text.trim().to_owned(),
        }))
    }

    fn read_cargo_config(root: &Path) -> Result<Option<TomlFile>, CheckError> {
        if root.join(".cargo/config").is_file() {
            return Self::read_toml(root, ".cargo/config");
        }
        Self::read_toml(root, ".cargo/config.toml")
    }

    fn is_nested_workspace(directory: &Path) -> bool {
        let Ok(text) = fs::read_to_string(directory.join("Cargo.toml")) else {
            return false;
        };
        parse_toml_file("Cargo.toml", &text)
            .is_ok_and(|file| file.root.find_entry("workspace").is_some())
    }

    fn walk_directory(
        root: &Path,
        relative: &str,
        is_skipped: &dyn Fn(&str, &Path) -> bool,
        is_recorded: &dyn Fn(&str, &Path) -> bool,
        files: &mut Vec<String>,
        directories: &mut Vec<String>,
    ) -> Result<(), CheckError> {
        let io_error = |error: std::io::Error| CheckError::Io {
            path: relative.to_owned(),
            source: error.into(),
        };
        let mut entries = fs::read_dir(root.join(relative))
            .map_err(io_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(io_error)?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let child = Self::join_relative(relative, &entry.file_name().to_string_lossy());
            let file_type = entry.file_type().map_err(io_error)?;
            if file_type.is_file() {
                files.push(child);
            } else if file_type.is_dir() && !is_skipped(&child, &entry.path()) {
                directories.push(child.clone());
                Self::walk_directory(root, &child, is_skipped, is_recorded, files, directories)?;
            } else if file_type.is_dir() && is_recorded(&child, &entry.path()) {
                directories.push(child);
            }
        }
        Ok(())
    }

    fn walk_workspace(
        root: &Path,
        target_directory: &Path,
        root_manifest: &TomlFile,
        members: &[Member],
        directories: &mut Vec<String>,
    ) -> Result<Vec<String>, CheckError> {
        let excluded: Vec<String> = root_manifest
            .root
            .find(&["workspace", "exclude"])
            .and_then(TomlValue::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(|value| {
                value
                    .as_str()
                    .map(|path| path.trim_end_matches('/').to_owned())
            })
            .collect();
        let member_directories: Vec<String> = members
            .iter()
            .map(|member| member.directory.clone())
            .collect();
        let is_unread = |relative: &str, path: &Path| -> bool {
            let is_excluded = excluded
                .iter()
                .any(|path| relative == path || relative.starts_with(&format!("{path}/")));
            let is_worktree = path.join(".git").is_file();
            path == target_directory
                || is_excluded
                || is_worktree
                || Self::is_nested_workspace(path)
        };
        let is_skipped = |relative: &str, path: &Path| -> bool {
            if is_member_source_directory(relative, &member_directories) {
                return false;
            }
            let name = relative.rsplit('/').next().unwrap_or(relative);
            is_skipped_directory_name(name) || is_unread(relative, path)
        };
        let is_recorded = |relative: &str, path: &Path| -> bool {
            let name = relative.rsplit('/').next().unwrap_or(relative);
            is_skipped_directory_name(name) && !is_unread(relative, path)
        };
        let mut files = Vec::new();
        Self::walk_directory(root, "", &is_skipped, &is_recorded, &mut files, directories)?;
        Ok(files)
    }
}

impl CargoWorkspaceReader {
    fn apply_dependency_names(
        mut member: Member,
        package: &Package,
        names: &BTreeMap<(String, String), String>,
    ) -> Member {
        for dependency in &mut member.dependencies {
            let renamed = package
                .dependencies
                .iter()
                .any(|declared| declared.name == dependency.name && declared.rename.is_some());
            if !renamed
                && let Some(name) = names.get(&(package.id.to_string(), dependency.name.clone()))
            {
                dependency.code_name = name.replace('-', "_");
            }
        }
        member
    }

    fn read_dependency_names(
        &self,
        dir: &Path,
        packages: &[&Package],
    ) -> BTreeMap<(String, String), String> {
        let options = vec!["--offline".to_owned(), "--locked".to_owned()];
        let resolved = MetadataCommand::new()
            .cargo_path(&self.cargo)
            .current_dir(dir)
            .other_options(options)
            .exec()
            .ok();
        let mut dependency_names = BTreeMap::new();
        for package in packages {
            for dependency in &package.dependencies {
                if let Some(provider) = packages.iter().find(|provider| {
                    provider.name.as_str() == dependency.name
                        && dependency.req.matches(&provider.version)
                }) && let Some(target) = provider.targets.iter().find(|target| {
                    target
                        .kind
                        .iter()
                        .any(|kind| kind.to_string() == "lib" || kind.to_string() == "rlib")
                }) {
                    dependency_names.insert(
                        (package.id.to_string(), dependency.name.clone()),
                        target.name.clone(),
                    );
                }
            }
        }
        let Some(resolved) = resolved else {
            return dependency_names;
        };
        let Some(graph) = &resolved.resolve else {
            return dependency_names;
        };
        for node in &graph.nodes {
            for dependency in &node.deps {
                if let Some(package) = resolved
                    .packages
                    .iter()
                    .find(|package| package.id == dependency.pkg)
                {
                    dependency_names.insert(
                        (node.id.to_string(), package.name.to_string()),
                        dependency.name.clone(),
                    );
                }
            }
        }
        dependency_names
    }
}

impl WorkspaceReader for CargoWorkspaceReader {
    fn read_workspace(&self, dir: &Path) -> Result<Workspace, CheckError> {
        let metadata = MetadataCommand::new()
            .cargo_path(&self.cargo)
            .current_dir(dir)
            .no_deps()
            .exec()
            .map_err(|error| CheckError::Metadata {
                source: error.into(),
            })?;
        let root = metadata.workspace_root.clone().into_std_path_buf();
        let target_directory = metadata.target_directory.clone().into_std_path_buf();
        let root_manifest = Self::read_manifest(&root, "Cargo.toml")?;
        let lock = Self::read_toml(&root, "Cargo.lock")?;
        let cargo_config = Self::read_cargo_config(&root)?;
        let packages = metadata.workspace_packages();
        let dependency_names = self.read_dependency_names(dir, &packages);
        let mut members = packages
            .iter()
            .map(|package| {
                Self::read_member(
                    &root,
                    package,
                    &packages,
                    &root_manifest,
                    lock.as_ref(),
                    cargo_config.as_ref(),
                )
                .map(|member| Self::apply_dependency_names(member, package, &dependency_names))
            })
            .collect::<Result<Vec<Member>, CheckError>>()?;
        members.sort_by(|left, right| left.directory.cmp(&right.directory));
        let mut directories = Vec::new();
        let files = Self::walk_workspace(
            &root,
            &target_directory,
            &root_manifest,
            &members,
            &mut directories,
        )?;
        Ok(Workspace {
            clippy_files: Self::read_clippy_files(&root, &members)?,
            rustfmt_files: Self::read_rustfmt_files(&root, &members)?,
            toolchain_file: Self::read_toolchain_file(&root)?,
            cargo_config,
            lock_file: lock,
            root,
            target_directory,
            root_manifest,
            members,
            files,
            directories,
        })
    }
}
