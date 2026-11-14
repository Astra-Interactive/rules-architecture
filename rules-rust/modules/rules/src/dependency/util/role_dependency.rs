use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::{CrateConfig, Role};
use crate::config::util::format_role_name;
use crate::source::model::SourceTree;
use crate::workspace::model::{Dependency, DependencyKind, Member, TomlValue};

const DIRECTION_SOURCE: &str = "arch-context.md, Direction and enforcement";

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
        if from == Some(Role::ProcMacro) {
            return false;
        }
        if dependency.features.iter().any(|feature| feature == "fake") {
            return dependency.kind == DependencyKind::Development;
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

    pub(crate) fn find_dependency_violation(
        &self,
        member: &Member,
        dependency: &Dependency,
        provider: &Member,
    ) -> Option<String> {
        if self.is_role_dependency_allowed(member, dependency, provider) {
            return None;
        }
        let from = self
            .find_role_config(&member.name)
            .and_then(|config| config.role);
        let to = self
            .find_role_config(&provider.name)
            .and_then(|config| config.role);
        let has_fake_feature = dependency.features.iter().any(|feature| feature == "fake");
        if has_fake_feature && from != Some(Role::ProcMacro) {
            return Some(format!(
                "{} enables the fake feature of {} outside [dev-dependencies] (arch-package.rs.md, Tests)",
                member.name, provider.name
            ));
        }
        let Some(from_role) = from else {
            return Some(format!(
                "{} has no role the linter detects: declare its role (README, Roles)",
                member.name
            ));
        };
        let has_no_row = matches!(
            from_role,
            Role::ProcMacro | Role::Core | Role::CoreApi | Role::Contract
        );
        if has_no_row {
            return Some(format!(
                "{} ({}) depends on no crate of the project (proc-macro, core, core-api, contract)",
                member.name,
                format_role_name(from_role)
            ));
        }
        let has_sibling_access = self
            .find_role_config(&member.name)
            .is_some_and(|config| config.implementation_access == "siblings");
        Some(format!(
            "{} ({}) may not depend on {} ({}): {} ({DIRECTION_SOURCE})",
            member.name,
            format_role_name(from_role),
            provider.name,
            to.map_or("no detected role", format_role_name),
            from_role.describe_requirement(to, has_sibling_access)
        ))
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
