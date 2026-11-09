use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::{CrateConfig, Role};
use crate::source::model::SourceTree;
use crate::workspace::model::{Dependency, DependencyKind, Member, TomlValue};

impl RuleContext {
    pub(crate) fn find_role_config(&self, name: &str) -> Option<&CrateConfig> {
        self.project_config
            .crates
            .iter()
            .find(|config| config.name == name)
    }

    pub(crate) fn is_plugin_product(&self) -> bool {
        self.workspace
            .root_manifest
            .root
            .find(&["workspace", "metadata", "klibs-rules", "product-kind"])
            .and_then(TomlValue::as_str)
            == Some("platform-plugin")
    }

    pub(crate) fn has_contract_access(&self, name: &str) -> bool {
        self.indexes
            .iter()
            .filter(|index| index.crate_name == name)
            .any(|index| {
                index.modules.iter().any(|module| {
                    let Some(word) = module.module.path.last() else {
                        return false;
                    };
                    module.tree == SourceTree::Src
                        && !module.is_file_module
                        && (word == "route" || self.is_technology_kind(name, word))
                })
            })
    }

    fn is_ui_dependency_allowed(&self, member: &Member, to: Option<Role>, siblings: bool) -> bool {
        let Some(config) = self.find_role_config(&member.name) else {
            return false;
        };
        matches!(
            to,
            Some(Role::Api | Role::CoreApi | Role::Core | Role::CoreUi)
        ) || (to == Some(Role::Impl) && siblings && config.implementation_access == "siblings")
            || (to == Some(Role::ImplUi) && config.composition == "host")
    }

    pub(crate) fn is_role_dependency_allowed(
        &self,
        member: &Member,
        dependency: &Dependency,
        provider: &Member,
    ) -> bool {
        let from = self
            .find_role_config(&member.name)
            .and_then(|config| config.role);
        let to = self
            .find_role_config(&provider.name)
            .and_then(|config| config.role);
        if [Some(Role::Tooling), Some(Role::ProcMacro)].contains(&from) || to == Some(Role::Tooling)
        {
            return false;
        }
        if dependency.kind == DependencyKind::Development {
            return true;
        }
        if dependency.features.iter().any(|feature| feature == "fake") {
            return false;
        }
        if to == Some(Role::ProcMacro) {
            return true;
        }
        if to == Some(Role::Contract) && self.has_contract_access(&member.name) {
            return true;
        }
        let parent = |directory: &str| {
            directory
                .rsplit_once('/')
                .map(|(parent, _)| parent.to_owned())
        };
        let siblings = parent(&member.directory) == parent(&provider.directory);
        match from {
            Some(Role::EntryPoint) => to != Some(Role::EntryPoint),
            Some(Role::Platform | Role::Context) => matches!(
                to,
                Some(
                    Role::Api
                        | Role::CoreApi
                        | Role::Core
                        | Role::CorePlatform
                        | Role::StorageContract
                )
            ),
            Some(Role::Api) => matches!(
                to,
                Some(Role::Api | Role::CoreApi | Role::Core | Role::StorageContract)
            ),
            Some(Role::Impl) => matches!(to, Some(Role::Api | Role::CoreApi | Role::Core)),
            Some(Role::ImplUi) => self.is_ui_dependency_allowed(member, to, siblings),
            Some(Role::Technology) => {
                matches!(to, Some(Role::StorageContract | Role::CoreApi | Role::Core))
                    || (to == Some(Role::Api) && siblings)
            }
            Some(Role::StorageContract) => matches!(to, Some(Role::CoreApi | Role::Core)),
            Some(Role::CorePlatform | Role::CoreUi) => to == Some(Role::CoreApi),
            _ => false,
        }
    }
}

pub(crate) fn build_dependency_finding(
    rule: &str,
    member: &Member,
    dependency: &Dependency,
    provider: &str,
    message: &str,
) -> RuleFinding {
    RuleFinding {
        finding: Finding {
            rule_id: RuleId(rule.to_owned()),
            location: Location {
                path: member.manifest.path.clone(),
                line: dependency.line,
            },
            subject: format!("{} -> {provider}", member.name),
            message: message.to_owned(),
        },
        anchor: Anchor::Edge {
            from_crate: member.name.clone(),
            to_crate: provider.to_owned(),
        },
    }
}
