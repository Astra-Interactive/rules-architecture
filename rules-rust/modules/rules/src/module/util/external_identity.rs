use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::source::model::ExternalPath;

impl RuleContext {
    pub(crate) fn resolve_external_identity(&self, ordinal: usize, path: &str) -> ExternalPath {
        self.resolve_external_identity_at(ordinal, path, 0)
    }

    fn resolve_external_identity_at(
        &self,
        ordinal: usize,
        path: &str,
        depth: usize,
    ) -> ExternalPath {
        let segments: Vec<String> = path.split("::").map(str::to_owned).collect();
        let root = segments.first().map(String::as_str).unwrap_or("");
        let owner = self
            .workspace
            .members
            .iter()
            .find(|member| member.name == self.indexes[ordinal].crate_name);
        let dependency = owner.and_then(|member| {
            member
                .dependencies
                .iter()
                .find(|dependency| dependency.code_name == root)
        });
        let package = dependency.map_or_else(
            || root.to_owned(),
            |dependency| dependency.name.replace('-', "_"),
        );
        let member = self.workspace.members.iter().position(|member| {
            dependency.is_some_and(|dependency| {
                dependency.path.as_ref().is_some_and(|path| {
                    member.directory == *path
                        || self.workspace.root.join(&member.directory) == std::path::Path::new(path)
                })
            })
        });
        let identity = ExternalPath {
            package,
            segments,
            member,
        };
        if depth >= 8 {
            return identity;
        }
        let Some(member) = identity.member else {
            return identity;
        };
        let provider = &self.workspace.members[member].name;
        let target = self
            .indexes
            .iter()
            .enumerate()
            .find(|(_, index)| index.crate_name == *provider);
        if let Some((target, index)) = target
            && let crate::source::model::Resolution::External(path) =
                index.resolve_segments(0, &identity.segments[1..], 0)
        {
            return self.resolve_external_identity_at(target, &path, depth + 1);
        }
        identity
    }

    pub(crate) fn is_platform_identity(&self, path: &ExternalPath) -> bool {
        if self.project_config.platform_words.is_empty() {
            return false;
        }
        let declared = path
            .member
            .and_then(|member| self.find_role_config(&self.workspace.members[member].name))
            .is_some_and(|config| matches!(config.role, Some(Role::Platform | Role::CorePlatform)));
        declared
            || self
                .project_config
                .platform_words
                .iter()
                .any(|word| Self::matches_crate_word(&path.package, word))
    }

    pub(crate) fn matches_crate_word(package: &str, word: &str) -> bool {
        let normalized = word.replace('-', "_");
        package == normalized || package.split('_').next() == Some(normalized.as_str())
    }

    pub(crate) fn is_technology_identity(&self, ordinal: usize, path: &ExternalPath) -> bool {
        let declared = path
            .member
            .and_then(|member| self.find_role_config(&self.workspace.members[member].name))
            .is_some_and(|config| config.role == Some(Role::Technology));
        if declared {
            return true;
        }
        let crate_name = &self.indexes[ordinal].crate_name;
        let configured =
            crate::module::util::list_shape_words(self, crate_name, "technology-words")
                .iter()
                .any(|word| Self::matches_crate_word(&path.package, word));
        if configured {
            return true;
        }
        self.indexes.iter().any(|index| {
            index.modules.iter().any(|module| {
                let Some(word) = module.module.path.last() else {
                    return false;
                };
                !module.is_file_module
                    && self.is_technology_kind(&index.crate_name, word)
                    && Self::matches_crate_word(&path.package, word)
            })
        })
    }
}
