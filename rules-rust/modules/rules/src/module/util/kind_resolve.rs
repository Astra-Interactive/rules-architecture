use crate::check::model::RuleContext;

impl RuleContext {
    pub(crate) fn has_technology_implementation_role(&self, name: &str) -> bool {
        self.find_role_config(name).is_some_and(|config| {
            matches!(
                config.role,
                Some(
                    crate::config::model::Role::Technology
                        | crate::config::model::Role::Platform
                        | crate::config::model::Role::CorePlatform
                )
            )
        })
    }

    pub(crate) fn find_technology_module(
        &self,
        ordinal: usize,
        mut module: usize,
    ) -> Option<usize> {
        let index = &self.indexes[ordinal];
        loop {
            let current = &index.modules[module];
            if !current.is_file_module
                && current
                    .module
                    .path
                    .last()
                    .is_some_and(|name| self.is_technology_kind(&index.crate_name, name))
            {
                return Some(module);
            }
            module = current.parent?;
        }
    }

    pub(crate) fn is_in_technology_module(&self, ordinal: usize, module: usize) -> bool {
        self.find_technology_module(ordinal, module).is_some()
    }

    pub(crate) fn is_entry_kind(&self, crate_name: &str, name: &str) -> bool {
        if [
            "command",
            "event",
            "route",
            "job",
            "service",
            "warmup",
            "migration",
            "error",
        ]
        .contains(&name)
        {
            return true;
        }
        let workspace = &self.workspace.root_manifest.root;
        let member = self
            .workspace
            .members
            .iter()
            .find(|member| member.name == crate_name);
        [
            workspace.find(&["workspace", "metadata", "klibs-rules", "entry-point-words"]),
            member.and_then(|member| {
                member.manifest.root.find(&[
                    "package",
                    "metadata",
                    "klibs-rules",
                    "entry-point-words",
                ])
            }),
        ]
        .into_iter()
        .flatten()
        .flat_map(|value| value.as_array().into_iter().flatten())
        .any(|value| value.as_str() == Some(name))
    }

    pub(crate) fn is_technology_kind(&self, crate_name: &str, name: &str) -> bool {
        self.project_config
            .crates
            .iter()
            .find(|config| config.name == crate_name)
            .is_some_and(|config| config.technology_words.iter().any(|word| word == name))
    }

    pub(crate) fn find_kind_module(&self, index: usize, mut module: usize) -> Option<usize> {
        let index = &self.indexes[index];
        loop {
            let current = index.find_module(module)?;
            let name = current.module.path.last().map(String::as_str).unwrap_or("");
            let fixed = [
                "api", "model", "usecase", "util", "di", "stub", "fake", "table", "entity",
                "database",
            ]
            .contains(&name);
            if !current.is_file_module
                && (fixed
                    || self.is_entry_kind(&index.crate_name, name)
                    || self.is_technology_kind(&index.crate_name, name))
            {
                return Some(module);
            }
            module = current.parent?;
        }
    }
}
