use crate::check::model::RuleContext;
use crate::module::model::{ModuleDependencyPath, ModuleIdentity};
use crate::module::util::owns_implementation;
use crate::module::util::resolved_di_type;
use crate::source::model::{CfgPredicate, WorkspaceResolvedPath};

fn dependencies(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> Vec<WorkspaceResolvedPath> {
    let index = &context.indexes[ordinal];
    let mut targets = Vec::new();
    for implementation in index
        .implementations
        .iter()
        .filter(|implementation| owns_implementation(index, identity, implementation))
    {
        for method in &implementation.syntax.items {
            let syn::ImplItem::Fn(method) = method else {
                continue;
            };
            if method.sig.ident != "new" {
                continue;
            }
            for input in &method.sig.inputs {
                let syn::FnArg::Typed(parameter) = input else {
                    continue;
                };
                let cfg = implementation
                    .cfg
                    .and(&CfgPredicate::from_attributes(&method.attrs));
                targets.extend(
                    context
                        .resolve_workspace_type(ordinal, implementation.module, &parameter.ty)
                        .into_iter()
                        .filter(|target| resolved_di_type(context, target))
                        .map(|mut target| {
                            target.path.cfg = target.path.cfg.and(&cfg);
                            target
                        }),
                );
            }
        }
    }
    targets
}

pub(crate) fn has_module_parameter_cycle(
    context: &RuleContext,
    ordinal: usize,
    identity: usize,
) -> bool {
    let start = ModuleIdentity {
        index: ordinal,
        item: identity,
    };
    let mut pending = vec![ModuleDependencyPath {
        identity: start,
        cfg: context.indexes[ordinal].items[identity].cfg.clone(),
        visited: vec![start],
    }];
    while let Some(state) = pending.pop() {
        for target in dependencies(context, state.identity.index, state.identity.item) {
            let Some(identity) = target.path.item else {
                continue;
            };
            let cfg = state.cfg.and(&target.path.cfg);
            if !cfg.is_satisfiable() {
                continue;
            }
            let node = ModuleIdentity {
                index: target.index,
                item: identity,
            };
            if node == start {
                return true;
            }
            if state.visited.contains(&node) {
                continue;
            }
            let mut path = state.visited.clone();
            path.push(node);
            pending.push(ModuleDependencyPath {
                identity: node,
                cfg,
                visited: path,
            });
        }
    }
    false
}
