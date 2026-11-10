use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::model::LifecycleFacts;
use crate::module::rule::{LifecycleBodyVisitor, LifecycleFields};
use crate::module::util::{build_shape_item_finding, owns_implementation};
use crate::module::util::{is_di_type, lifecycle_implementation, resolved_di_type};
use syn::{ImplItem, Item};

pub(crate) struct DiModuleLifecycle;

impl DiModuleLifecycle {
    fn required_trait_method(member: &syn::TraitItem) -> Option<String> {
        match member {
            syn::TraitItem::Fn(method)
                if method
                    .default
                    .as_ref()
                    .is_none_or(|body| !body.stmts.is_empty()) =>
            {
                Some(method.sig.ident.to_string())
            }
            _ => None,
        }
    }

    fn lifecycle_methods(
        context: &RuleContext,
        target: &crate::source::model::WorkspaceResolvedPath,
    ) -> std::collections::BTreeSet<String> {
        let Some(identity) = target.path.item else {
            return std::collections::BTreeSet::new();
        };
        let index = &context.indexes[target.index];
        let mut names = std::collections::BTreeSet::new();
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            let Some((path, _)) = &implementation.syntax.trait_ else {
                continue;
            };
            if !crate::module::util::lifecycle_trait_path(
                context,
                target.index,
                implementation.module,
                path,
            ) {
                continue;
            }
            names.extend(
                implementation
                    .syntax
                    .items
                    .iter()
                    .filter_map(|member| match member {
                        ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
                        _ => None,
                    }),
            );
            for contract in
                context.resolve_workspace_path(target.index, implementation.module, path)
            {
                let Some(identity) = contract.path.item else {
                    continue;
                };
                if let Item::Trait(contract) =
                    &context.indexes[contract.index].items[identity].syntax
                {
                    names.extend(
                        contract
                            .items
                            .iter()
                            .filter_map(Self::required_trait_method),
                    );
                }
            }
        }
        names
    }

    fn missing_lifecycle_method(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        owned: &[String],
    ) -> bool {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return false;
        };
        let required: std::collections::BTreeSet<_> = structure
            .fields
            .iter()
            .filter(|field| {
                field
                    .ident
                    .as_ref()
                    .is_some_and(|name| owned.contains(&name.to_string()))
            })
            .flat_map(|field| context.resolve_workspace_type(ordinal, item.module, &field.ty))
            .flat_map(|target| Self::lifecycle_methods(context, &target))
            .collect();
        let provided: std::collections::BTreeSet<_> = index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
            .flat_map(|implementation| &implementation.syntax.items)
            .filter_map(|member| match member {
                ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
                _ => None,
            })
            .collect();
        !required.is_subset(&provided)
    }

    fn child_dependencies(
        context: &RuleContext,
        target: &crate::source::model::WorkspaceResolvedPath,
    ) -> Vec<crate::source::model::WorkspaceResolvedPath> {
        let Some(identity) = target.path.item else {
            return Vec::new();
        };
        let index = &context.indexes[target.index];
        let mut dependencies = Vec::new();
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            let parameters = implementation
                .syntax
                .items
                .iter()
                .filter_map(|member| match member {
                    ImplItem::Fn(function) if function.sig.ident == "new" => {
                        Some(&function.sig.inputs)
                    }
                    _ => None,
                })
                .flatten();
            for parameter in parameters {
                if let syn::FnArg::Typed(parameter) = parameter {
                    dependencies.extend(context.resolve_workspace_type(
                        target.index,
                        implementation.module,
                        &parameter.ty,
                    ));
                }
            }
        }
        dependencies
    }

    fn matches_dependency(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        dependencies: &[crate::source::model::WorkspaceResolvedPath],
    ) -> bool {
        context
            .resolve_workspace_type(ordinal, module, typed)
            .iter()
            .any(|field| {
                dependencies.iter().any(|parameter| {
                    field.index == parameter.index && field.path.item == parameter.path.item
                })
            })
    }

    fn path_order_invalid(path: &[String], child: &str, dependency: &str, method: &str) -> bool {
        let child = path.iter().position(|name| name == child);
        let dependency = path.iter().position(|name| name == dependency);
        let (Some(child), Some(dependency)) = (child, dependency) else {
            return false;
        };
        if method == "on_disable" {
            child > dependency
        } else {
            dependency > child
        }
    }

    fn invalid_order(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        method: &str,
        paths: &[Vec<String>],
    ) -> bool {
        let item = &context.indexes[ordinal].items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return false;
        };
        for child in &structure.fields {
            let Some(child_name) = child.ident.as_ref().map(ToString::to_string) else {
                continue;
            };
            let dependencies: Vec<_> = context
                .resolve_workspace_type(ordinal, item.module, &child.ty)
                .iter()
                .flat_map(|target| Self::child_dependencies(context, target))
                .collect();
            for dependency in &structure.fields {
                let Some(dependency_name) = dependency.ident.as_ref().map(ToString::to_string)
                else {
                    continue;
                };
                if dependency_name == child_name {
                    continue;
                }
                let required = Self::matches_dependency(
                    context,
                    ordinal,
                    item.module,
                    &dependency.ty,
                    &dependencies,
                );
                if !required {
                    continue;
                }
                let wrong = paths.iter().any(|path| {
                    Self::path_order_invalid(path, &child_name, &dependency_name, method)
                });
                if wrong {
                    return true;
                }
            }
        }
        false
    }

    fn is_exclusive_pointer(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        field: &syn::Field,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        context
            .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, cfg)
            .iter()
            .any(|path| {
                ["std", "alloc", "core"].contains(&path.package.as_str())
                    && path
                        .segments
                        .last()
                        .is_some_and(|name| ["Box", "Rc"].contains(&name.as_str()))
            })
    }

    fn add_field(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        field: &syn::Field,
        facts: &mut LifecycleFacts,
    ) {
        let cfg = item
            .cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                &field.attrs,
            ));
        if !cfg.is_satisfiable() {
            return;
        }
        let di = is_di_type(context, ordinal, item);
        let targets: Vec<_> = context
            .resolve_workspace_type(ordinal, item.module, &field.ty)
            .into_iter()
            .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
            .collect();
        let child = targets
            .iter()
            .any(|target| resolved_di_type(context, target));
        let scope = targets.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                context.indexes[target.index].items[identity].name == "TaskScope"
            })
        });
        let by_value = context
            .resolve_type_syntax_under(ordinal, item.module, &field.ty, &cfg)
            .iter()
            .all(|typed| matches!(typed.typed, syn::Type::Path(_)))
            && context
                .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                .is_empty();
        let exclusive_pointer =
            scope && Self::is_exclusive_pointer(context, ordinal, item, field, &cfg);
        facts.invalid |=
            (di && child && !by_value) || (!di && scope && by_value) || exclusive_pointer;
        let Some(name) = &field.ident else {
            return;
        };
        if child && by_value {
            facts.owned.push(name.to_string());
        }
        if scope && by_value {
            facts.scopes.push(name.to_string());
        }
        facts.needs_lifecycle |= child
            && by_value
            && targets.iter().any(|target| {
                target.path.item.is_some_and(|identity| {
                    lifecycle_implementation(context, target.index, identity)
                })
            });
    }

    fn fields(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        structure: &syn::ItemStruct,
    ) -> LifecycleFacts {
        let item = &context.indexes[ordinal].items[identity];
        let mut facts = LifecycleFacts {
            owned: Vec::new(),
            scopes: Vec::new(),
            aborted: std::collections::BTreeSet::new(),
            invalid: false,
            needs_lifecycle: false,
        };
        for field in &structure.fields {
            Self::add_field(context, ordinal, item, field, &mut facts);
        }
        facts
    }

    fn parameters(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        method: &syn::ImplItemFn,
    ) -> bool {
        if method.sig.ident != "new" {
            return false;
        }
        method.sig.inputs.iter().any(|parameter| {
            let syn::FnArg::Typed(parameter) = parameter else { return false; };
            let module_type = context.resolve_workspace_type(ordinal, module, &parameter.ty).iter().any(|target| resolved_di_type(context, target));
            let shared = context.resolve_type_syntax(ordinal, module, &parameter.ty).iter().all(|typed| matches!(&typed.typed, syn::Type::Reference(reference) if reference.mutability.is_none()));
            module_type && !shared
        })
    }

    fn list_shared_fields(context: &RuleContext, ordinal: usize, identity: usize) -> Vec<String> {
        let item = &context.indexes[ordinal].items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return Vec::new();
        };
        structure
            .fields
            .iter()
            .filter_map(|field| {
                let name = field.ident.as_ref()?;
                let cfg = item
                    .cfg
                    .and(&crate::source::model::CfgPredicate::from_attributes(
                        &field.attrs,
                    ));
                let shared = context
                    .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                    .iter()
                    .any(|path| {
                        ["std", "alloc"].contains(&path.package.as_str())
                            && path.segments.last().is_some_and(|segment| segment == "Arc")
                    });
                shared.then(|| name.to_string())
            })
            .collect()
    }

    fn list_task_scope_fields(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
    ) -> Vec<String> {
        let item = &context.indexes[ordinal].items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return Vec::new();
        };
        structure
            .fields
            .iter()
            .filter_map(|field| {
                let name = field.ident.as_ref()?;
                let scope = context
                    .resolve_workspace_type(ordinal, item.module, &field.ty)
                    .iter()
                    .any(|target| {
                        target.path.item.is_some_and(|identity| {
                            context.indexes[target.index].items[identity].name == "TaskScope"
                        })
                    });
                scope.then(|| name.to_string())
            })
            .collect()
    }

    fn methods(context: &RuleContext, ordinal: usize, identity: usize, facts: &mut LifecycleFacts) {
        let index = &context.indexes[ordinal];
        let shared = Self::list_shared_fields(context, ordinal, identity);
        let task_scopes = Self::list_task_scope_fields(context, ordinal, identity);
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            for method in &implementation.syntax.items {
                let ImplItem::Fn(method) = method else {
                    continue;
                };
                if !implementation
                    .cfg
                    .and(&crate::source::model::CfgPredicate::from_attributes(
                        &method.attrs,
                    ))
                    .is_satisfiable()
                {
                    continue;
                }
                facts.invalid |= ["start", "reload", "stop"]
                    .iter()
                    .any(|name| method.sig.ident == *name);
                facts.invalid |= Self::parameters(context, ordinal, implementation.module, method);
                if !["on_enable", "on_disable", "on_reload"]
                    .iter()
                    .any(|name| method.sig.ident == *name)
                {
                    continue;
                }
                let name = method.sig.ident.to_string();
                facts.invalid |= method.sig.asyncness.is_some();
                let body = LifecycleBodyVisitor::inspect(
                    context,
                    ordinal,
                    implementation.module,
                    &method.block,
                    &name,
                    LifecycleFields {
                        owned: &facts.owned,
                        scopes: &facts.scopes,
                        task_scopes: &task_scopes,
                        shared: &shared,
                    },
                );
                facts.invalid |= body.invalid;
                facts.invalid |= facts
                    .owned
                    .iter()
                    .any(|child| !body.forwarded.contains(child));
                facts.invalid |=
                    Self::invalid_order(context, ordinal, identity, &name, &body.paths);
                facts.needs_lifecycle |= body.child_lifecycle;
                facts.aborted.extend(body.aborted);
            }
        }
    }

    fn check_item(context: &RuleContext, ordinal: usize, identity: usize) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return None;
        };
        let mut facts = Self::fields(context, ordinal, identity, structure);
        if is_di_type(context, ordinal, item) {
            facts.invalid |=
                crate::module::util::has_module_parameter_cycle(context, ordinal, identity);
            Self::methods(context, ordinal, identity, &mut facts);
            facts.invalid |=
                Self::missing_lifecycle_method(context, ordinal, identity, &facts.owned);
            facts.invalid |= facts
                .scopes
                .iter()
                .any(|scope| !facts.aborted.contains(scope));
            facts.invalid |= (!facts.scopes.is_empty() || facts.needs_lifecycle)
                && !lifecycle_implementation(context, ordinal, identity);
        }
        facts.invalid.then(|| build_shape_item_finding(index, item, "DiModuleLifecycle", "DI lifecycle is synchronous, manages only owned children, receives modules by shared reference, and aborts owned task scopes on disable"))
    }
}

impl Rule for DiModuleLifecycle {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                (0..index.items.len())
                    .filter_map(move |identity| Self::check_item(context, ordinal, identity))
            })
            .collect()
    }
}
