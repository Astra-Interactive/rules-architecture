use crate::check::model::RuleContext;
use crate::module::util::find_kind_name;
use crate::source::model::IndexedItem;

pub(crate) fn is_di_type(context: &RuleContext, ordinal: usize, item: &IndexedItem) -> bool {
    find_kind_name(context, ordinal, item.module) == "di"
        && (item.name.ends_with("Module") || item.name == context.root_module_name())
}

pub(crate) fn resolved_di_type(
    context: &RuleContext,
    target: &crate::source::model::WorkspaceResolvedPath,
) -> bool {
    target.path.item.is_some_and(|identity| {
        is_di_type(
            context,
            target.index,
            &context.indexes[target.index].items[identity],
        )
    })
}

pub(crate) fn is_construction_use(
    context: &RuleContext,
    usage: &crate::source::model::WorkspaceUse,
) -> bool {
    use crate::source::model::WorkspaceUseKind;
    match usage.kind {
        WorkspaceUseKind::Construction => true,
        WorkspaceUseKind::Call => usage.path.last().is_some_and(|name| name == "new"),
        WorkspaceUseKind::Value => {
            matches!(&context.indexes[usage.target_index].items[usage.target_item].syntax, syn::Item::Struct(value) if matches!(value.fields, syn::Fields::Unit))
        }
        _ => false,
    }
}

pub(crate) fn workspace_implementation_under(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
    cfg: &crate::source::model::CfgPredicate,
) -> bool {
    let index = &context.indexes[ordinal];
    index.implementations.iter().any(|implementation| {
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        crate::module::util::owns_implementation(index, identity, implementation)
            && context
                .resolve_workspace_path(ordinal, implementation.module, path)
                .iter()
                .any(|target| {
                    cfg.in_scope(ordinal)
                        .and(&implementation.cfg.in_scope(ordinal))
                        .and(&target.path.cfg.in_scope(target.index))
                        .is_satisfiable()
                        && target.path.item.is_some_and(|item| {
                            matches!(
                                context.indexes[target.index].items[item].syntax,
                                syn::Item::Trait(_)
                            )
                        })
                })
    })
}

pub(crate) fn workspace_implementation(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> bool {
    workspace_implementation_under(
        context,
        ordinal,
        identity,
        &context.indexes[ordinal].items[identity].cfg,
    )
}

pub(crate) fn lifecycle_trait_path(
    context: &RuleContext,
    ordinal: usize,
    module: usize,
    path: &syn::Path,
) -> bool {
    let configured = context.workspace.root_manifest.root.find(&[
        "workspace",
        "metadata",
        "klibs-rules",
        "lifecycle-traits",
    ]);
    let matches = |name: &str| match configured.map(|value| &value.kind) {
        Some(crate::workspace::model::TomlKind::Array(names)) => names.iter().any(|value| {
            value
                .as_str()
                .is_some_and(|configured| configured.rsplit("::").next() == Some(name))
        }),
        _ => name == "Lifecycle",
    };
    let workspace = context.resolve_workspace_path(ordinal, module, path);
    if !workspace.is_empty() {
        return workspace.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                matches(&context.indexes[target.index].items[identity].name)
            })
        });
    }
    match context.indexes[ordinal].resolve_path(module, path) {
        crate::source::model::Resolution::External(path) => {
            path.rsplit("::").next().is_some_and(matches)
        }
        _ => false,
    }
}

pub(crate) fn lifecycle_implementation(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> bool {
    let index = &context.indexes[ordinal];
    index.implementations.iter().any(|implementation| {
        if !crate::module::util::owns_implementation(index, identity, implementation) {
            return false;
        }
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        lifecycle_trait_path(context, ordinal, implementation.module, path)
    })
}
