use crate::check::model::RuleContext;
use crate::config::util::{FIXED_ENTRY_WORDS, FIXED_KIND_WORDS};
use crate::module::util::list_shape_words;

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
        FIXED_ENTRY_WORDS.contains(&name)
            || list_shape_words(self, crate_name, "entry-point-words")
                .iter()
                .any(|word| word == name)
    }

    pub(crate) fn is_role_kind(&self, crate_name: &str, name: &str) -> bool {
        list_shape_words(self, crate_name, "role-kinds")
            .iter()
            .any(|word| word == name)
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
            if !current.is_file_module
                && (FIXED_KIND_WORDS.contains(&name)
                    || self.is_entry_kind(&index.crate_name, name)
                    || self.is_technology_kind(&index.crate_name, name)
                    || self.is_role_kind(&index.crate_name, name))
            {
                return Some(module);
            }
            module = current.parent?;
        }
    }
}
