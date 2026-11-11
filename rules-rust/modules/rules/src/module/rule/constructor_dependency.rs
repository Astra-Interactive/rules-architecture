use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::module::util::find_kind_name;
use crate::module::util::{is_di_type, workspace_implementation_under};
use crate::source::model::SourceTree;
use crate::source::model::WorkspaceUse;

pub(crate) struct ConstructorDependency;

impl ConstructorDependency {
    fn is_entry_point_crate(context: &RuleContext, ordinal: usize) -> bool {
        context.project_config.crates.iter().any(|config| {
            config.name == context.indexes[ordinal].crate_name
                && config.role == Some(Role::EntryPoint)
        })
    }

    fn is_entry_root_function(context: &RuleContext, usage: &WorkspaceUse) -> bool {
        let index = &context.indexes[usage.source_index];
        usage.source_module == 0
            && Self::is_entry_point_crate(context, usage.source_index)
            && !index.implementations.iter().any(|implementation| {
                implementation.file == usage.file && implementation.span.contains(&usage.span)
            })
    }

    fn is_outside_free_function(context: &RuleContext, usage: &WorkspaceUse) -> bool {
        let source = &context.indexes[usage.source_index].modules[usage.source_module];
        let declaration = context.indexes[usage.target_index].items[usage.target_item].module;
        let is_beside_type =
            usage.source_index == usage.target_index && usage.source_module == declaration;
        source.tree == SourceTree::Src && !is_beside_type
    }

    fn inside_constructor(context: &RuleContext, usage: &WorkspaceUse) -> bool {
        let Some(function) = usage.function.as_deref() else {
            return false;
        };
        let index = &context.indexes[usage.source_index];
        let implementations: Vec<_> = index
            .implementations
            .iter()
            .filter(|implementation| {
                implementation.file == usage.file && implementation.span.contains(&usage.span)
            })
            .collect();
        let own_value = implementations.iter().any(|implementation| {
            context
                .resolve_workspace_type(
                    usage.source_index,
                    implementation.module,
                    &implementation.syntax.self_ty,
                )
                .iter()
                .any(|target| {
                    target.index == usage.target_index
                        && target.path.item == Some(usage.target_item)
                })
        });
        if own_value {
            return false;
        }
        let root = implementations.iter().any(|implementation| {
            context
                .resolve_workspace_type(
                    usage.source_index,
                    implementation.module,
                    &implementation.syntax.self_ty,
                )
                .iter()
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        is_di_type(
                            context,
                            target.index,
                            &context.indexes[target.index].items[identity],
                        )
                    })
                })
        });
        let entry_root_function =
            implementations.is_empty() && Self::is_entry_root_function(context, usage);
        let outside_free_function =
            implementations.is_empty() && Self::is_outside_free_function(context, usage);
        !root
            && (!implementations.is_empty()
                || entry_root_function
                || outside_free_function
                || function == "new"
                || function.starts_with("with_")
                || function.starts_with("from_")
                || function == "default")
    }
}

impl Rule for ConstructorDependency {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let names = context.name_index();
        names
            .uses
            .iter()
            .filter(|usage| {
                usage.block.is_some()
                    && !usage.is_test
                    && !usage.ambiguous
                    && usage.cfg.is_satisfiable()
            })
            .filter(|usage| crate::module::util::is_construction_use(context, usage))
            .filter(|usage| {
                !is_di_type(
                    context,
                    usage.target_index,
                    &context.indexes[usage.target_index].items[usage.target_item],
                )
            })
            .filter(|usage| {
                find_kind_name(context, usage.source_index, usage.source_module) != "di"
            })
            .filter(|usage| {
                workspace_implementation_under(
                    context,
                    usage.target_index,
                    usage.target_item,
                    &usage.cfg,
                )
            })
            .filter(|usage| Self::inside_constructor(context, usage))
            .map(|usage| {
                build_node_finding(
                    "ConstructorDependency",
                    &usage.file,
                    usage.anchor,
                    context.indexes[usage.target_index].items[usage.target_item]
                        .name
                        .clone(),
                    "receive the workspace dependency through the constructor; assemble it in DI"
                        .to_owned(),
                )
            })
            .collect()
    }
}
