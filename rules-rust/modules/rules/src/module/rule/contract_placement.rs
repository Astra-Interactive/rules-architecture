use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::module::rule::KindImportDirection;
use crate::module::util::{build_shape_item_finding, find_kind_name, implements_crate_trait};
use crate::source::model::{CrateIndex, IndexedImpl, IndexedItem, Resolution};
use syn::Item;

pub(crate) struct ContractPlacement;

impl ContractPlacement {
    fn is_wrong_default_usecase(context: &RuleContext, ordinal: usize, item: &IndexedItem) -> bool {
        if !matches!(item.syntax, Item::Struct(_))
            || !item.name.starts_with("Default")
            || context.is_plugin_product()
        {
            return false;
        }
        let index = &context.indexes[ordinal];
        index
            .implementations
            .iter()
            .filter(|implementation| {
                implements_crate_trait(index, item, implementation)
                    || crate::module::util::resolve_shape_type(
                        index,
                        implementation.module,
                        &implementation.syntax.self_ty,
                    )
                    .iter()
                    .any(|target| std::ptr::eq(&index.items[*target], item))
            })
            .any(|implementation| {
                let Some((path, _)) = &implementation.syntax.trait_ else {
                    return false;
                };
                context
                    .resolve_workspace_path(ordinal, implementation.module, path)
                    .iter()
                    .any(|resolved| {
                        let target_index = &context.indexes[resolved.index];
                        let Some(target) = resolved.path.item.map(|item| &target_index.items[item])
                        else {
                            return false;
                        };
                        if !matches!(target.syntax, Item::Trait(_))
                            || !target.name.ends_with("UseCase")
                        {
                            return false;
                        }
                        let role = context
                            .find_role_config(&index.crate_name)
                            .and_then(|config| config.role);
                        let member = context
                            .workspace
                            .members
                            .iter()
                            .find(|member| member.name == index.crate_name);
                        let owner = context
                            .workspace
                            .members
                            .iter()
                            .find(|member| member.name == target_index.crate_name);
                        let siblings = member.zip(owner).is_some_and(|(member, owner)| {
                            member.directory.rsplit_once('/').map(|(parent, _)| parent)
                                == owner.directory.rsplit_once('/').map(|(parent, _)| parent)
                        });
                        role != Some(Role::Impl)
                            || find_kind_name(context, ordinal, item.module) != "usecase"
                            || !siblings
                    })
            })
    }

    pub(crate) fn is_state_holder_kind(
        context: &RuleContext,
        crate_name: &str,
        kind: &str,
    ) -> bool {
        let is_fixed_kind = KindImportDirection::FIXED_ENTRY_KINDS.contains(&kind);
        !is_fixed_kind && context.is_entry_kind(crate_name, kind)
    }

    pub(crate) fn pascal_kind(kind: &str) -> String {
        kind.split('_')
            .filter(|word| !word.is_empty())
            .map(|word| {
                let mut letters = word.chars();
                letters
                    .next()
                    .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                    .unwrap_or_default()
            })
            .collect()
    }

    pub(crate) fn is_app_api_state_holder(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let kind = find_kind_name(context, ordinal, item.module);
        let role = context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role);
        matches!(item.syntax, Item::Trait(_))
            && !context.is_plugin_product()
            && role == Some(Role::Api)
            && Self::is_state_holder_kind(context, &index.crate_name, kind)
            && item.name.ends_with(&Self::pascal_kind(kind))
    }

    fn is_implementation_of(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        item: &IndexedItem,
    ) -> bool {
        implementation
            .syntax
            .trait_
            .as_ref()
            .is_some_and(
                |(path, _)| match index.resolve_path(implementation.module, path) {
                    Resolution::Internal(paths) => paths
                        .iter()
                        .filter_map(|path| path.item)
                        .any(|target| std::ptr::eq(&index.items[target], item)),
                    _ => false,
                },
            )
    }

    fn has_default_beside(context: &RuleContext, ordinal: usize, item: &IndexedItem) -> bool {
        let index = &context.indexes[ordinal];
        let kind = context.find_kind_module(ordinal, item.module);
        let holder = Self::is_state_holder_kind(
            context,
            &index.crate_name,
            find_kind_name(context, ordinal, item.module),
        );
        index.items.iter().any(|candidate| {
            let beside = candidate.file == item.file
                || (holder
                    && kind.is_some()
                    && context.find_kind_module(ordinal, candidate.module) == kind);
            beside
                && candidate.name == format!("Default{}", item.name)
                && index.implementations.iter().any(|implementation| {
                    Self::is_implementation_of(index, implementation, item)
                        && implements_crate_trait(index, candidate, implementation)
                })
        })
    }

    fn check_item(context: &RuleContext, ordinal: usize, item: &IndexedItem) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let kind = find_kind_name(context, ordinal, item.module);
        let mut findings = Vec::new();
        let misplaced_trait = matches!(&item.syntax, Item::Trait(contract) if kind != "api" && !Self::is_exception(context, ordinal, item, contract));
        if misplaced_trait {
            findings.push(build_shape_item_finding(index, item, "ContractPlacement", "a crate contract lives in api unless its documented factory, extension, use-case, state-holder or registration exception applies"));
        }
        if Self::is_wrong_default_usecase(context, ordinal, item) {
            findings.push(build_shape_item_finding(index, item, "ContractPlacement", "an app use-case Default implementation lives in its owner's sibling impl/usecase unit"));
        }
        let valid_stub = matches!(item.syntax, Item::Struct(_))
            && index
                .implementations
                .iter()
                .any(|implementation| implements_crate_trait(index, item, implementation));
        if kind == "stub" && !valid_stub {
            findings.push(build_shape_item_finding(
                index,
                item,
                "ContractPlacement",
                "stub holds structs implementing the crate's contracts",
            ));
        }
        findings
    }

    fn check_impl(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Option<RuleFinding> {
        if find_kind_name(context, ordinal, implementation.module) != "api" {
            return None;
        }
        let index = &context.indexes[ordinal];
        let (path, _) = implementation.syntax.trait_.as_ref()?;
        let internal = match index.resolve_path(implementation.module, path) {
            Resolution::Internal(paths) => {
                paths.iter().filter_map(|path| path.item).any(|target| {
                    matches!(index.items[target].syntax, Item::Trait(_))
                        && index.items[target]
                            .cfg
                            .and(&implementation.cfg)
                            .is_satisfiable()
                })
            }
            _ => false,
        };
        internal.then(|| {
            build_node_finding(
                "ContractPlacement",
                &implementation.file,
                implementation.span,
                index.modules[implementation.module].module.path.join("::"),
                "a crate contract's implementation lives outside api".to_owned(),
            )
        })
    }

    fn is_exception(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        _contract: &syn::ItemTrait,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let kind = find_kind_name(context, ordinal, item.module);
        let factory = item.name.strip_suffix("Factory").is_some_and(|owner| {
            index.items.iter().any(|candidate| {
                candidate.file == item.file
                    && candidate.name == owner
                    && matches!(
                        candidate.syntax,
                        Item::Struct(_) | Item::Enum(_) | Item::Trait(_)
                    )
                    && candidate.cfg.and(&item.cfg).is_satisfiable()
            })
        });
        let extension = item.name.ends_with("Ext") && kind == "util";
        let usecase = item.name.ends_with("UseCase")
            && kind == "usecase"
            && !context.is_plugin_product()
            && context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                == Some(Role::Api);
        let entry = context.is_entry_kind(&index.crate_name, kind);
        let state_holder = (entry && Self::has_default_beside(context, ordinal, item))
            || Self::is_app_api_state_holder(context, ordinal, item);
        let is_explicit_kind = ["model", "usecase", "util"].contains(&kind);
        let mixed_core =
            !is_explicit_kind
                && context.project_config.crates.iter().any(|config| {
                    config.name == index.crate_name && config.role == Some(Role::Core)
                });
        factory
            || extension
            || usecase
            || state_holder
            || crate::module::util::is_di_type(context, ordinal, item)
            || ((entry || mixed_core) && context.has_registration_shape(ordinal, item))
    }
}

impl Rule for ContractPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(
                index
                    .items
                    .iter()
                    .flat_map(|item| Self::check_item(context, ordinal, item)),
            );
            findings.extend(
                index.implementations.iter().filter_map(|implementation| {
                    Self::check_impl(context, ordinal, implementation)
                }),
            );
        }
        findings
    }
}
