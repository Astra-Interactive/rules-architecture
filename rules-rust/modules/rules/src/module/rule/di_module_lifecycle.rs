use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::config::model::Role;
use crate::module::model::LifecycleFacts;
use crate::module::rule::{
    LifecycleBodyVisitor, LifecycleFields, ScopeCallVisitor, ScopeOwnership,
};
use crate::module::util::{build_shape_item_finding, owns_implementation};
use crate::module::util::{is_di_type, lifecycle_implementation, resolved_di_type};
use crate::source::model::{CrateIndex, IndexedItem, SourceTree};
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

    fn record_scope(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        shape: &ScopeShape,
        field: &syn::Field,
        facts: &mut LifecycleFacts,
    ) -> bool {
        let Some(name) = field.ident.as_ref().map(ToString::to_string) else {
            return !shape.is_di && shape.is_by_value;
        };
        let is_built = ScopeOwnership::is_built_by_constructors(context, ordinal, identity, &name);
        if shape.is_by_value {
            facts.scopes.push(name.clone());
            let owns_it =
                is_built && ScopeOwnership::aborts_in_owner(context, ordinal, identity, &name);
            return !shape.is_di && !owns_it;
        }
        if shape.is_di && !shape.is_exclusive_pointer && is_built {
            facts.scopes.push(name);
        }
        false
    }

    fn add_field(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
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
        let scope_shape = ScopeShape {
            is_di: di,
            is_by_value: by_value,
            is_exclusive_pointer: exclusive_pointer,
        };
        let is_scope_misplaced =
            scope && Self::record_scope(context, ordinal, identity, &scope_shape, field, facts);
        facts.invalid |= (di && child && !by_value) || is_scope_misplaced || exclusive_pointer;
        let Some(name) = &field.ident else {
            return;
        };
        if child && by_value {
            facts.owned.push(name.to_string());
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
            Self::add_field(context, ordinal, identity, item, field, &mut facts);
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

    fn is_technology_module(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        let is_technology_crate = matches!(
            context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role),
            Some(Role::Technology | Role::Platform | Role::CorePlatform)
        );
        is_technology_crate
            || index.modules[index.items[identity].module]
                .module
                .path
                .iter()
                .any(|segment| context.is_technology_kind(&index.crate_name, segment))
    }

    fn list_runtime_handle_fields(
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
                let cfg = item
                    .cfg
                    .and(&crate::source::model::CfgPredicate::from_attributes(
                        &field.attrs,
                    ));
                let is_handle = context
                    .resolve_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                    .iter()
                    .any(|path| {
                        path.package == "tokio"
                            && path
                                .segments
                                .ends_with(&["runtime".to_owned(), "Handle".to_owned()])
                    });
                is_handle.then(|| name.to_string())
            })
            .collect()
    }

    fn list_handed_fields(context: &RuleContext, ordinal: usize, identity: usize) -> Vec<String> {
        let item = &context.indexes[ordinal].items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return Vec::new();
        };
        structure
            .fields
            .iter()
            .enumerate()
            .filter_map(|(position, field)| {
                let cfg = item
                    .cfg
                    .and(&crate::source::model::CfgPredicate::from_attributes(
                        &field.attrs,
                    ));
                let is_scope = context
                    .resolve_workspace_type(ordinal, item.module, &field.ty)
                    .iter()
                    .any(|target| {
                        target.path.item.is_some_and(|identity| {
                            context.indexes[target.index].items[identity].name == "TaskScope"
                        })
                    });
                let is_arc = context
                    .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                    .iter()
                    .any(|path| {
                        ["std", "alloc"].contains(&path.package.as_str())
                            && path.segments.last().is_some_and(|segment| segment == "Arc")
                    });
                (is_scope && is_arc).then(|| {
                    field
                        .ident
                        .as_ref()
                        .map_or_else(|| position.to_string(), ToString::to_string)
                })
            })
            .collect()
    }

    fn check_handed_scope(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
    ) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        if is_di_type(context, ordinal, item) || index.modules[item.module].tree != SourceTree::Src
        {
            return None;
        }
        let handed = Self::list_handed_fields(context, ordinal, identity);
        if handed.is_empty() {
            return None;
        }
        let stopped: Vec<String> = index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
            .flat_map(|implementation| &implementation.syntax.items)
            .filter_map(|member| match member {
                ImplItem::Fn(method) => Some(ScopeCallVisitor::collect(&method.block)),
                _ => None,
            })
            .flatten()
            .filter(|call| {
                handed.contains(&call.field) && ["abort", "reload"].contains(&call.method.as_str())
            })
            .map(|call| call.field)
            .collect();
        if stopped.is_empty() {
            return None;
        }
        let is_built_by_the_type = stopped.iter().all(|field| {
            ScopeOwnership::is_built_by_constructors(context, ordinal, identity, field)
        });
        let message = if is_built_by_the_type {
            Self::BUILT_SCOPE_MESSAGE
        } else {
            Self::HANDED_SCOPE_MESSAGE
        };
        Some(build_shape_item_finding(
            index,
            item,
            "DiModuleLifecycle",
            message,
        ))
    }

    fn find_root(parents: &[usize], start: usize) -> usize {
        let mut node = start;
        while parents[node] != node {
            node = parents[node];
        }
        node
    }

    fn list_product_roots(context: &RuleContext) -> Vec<usize> {
        let members = &context.workspace.members;
        let mut parents: Vec<usize> = (0..members.len()).collect();
        for (position, member) in members.iter().enumerate() {
            for dependency in &member.dependencies {
                let Some(path) = &dependency.path else {
                    continue;
                };
                let Some(target) = members.iter().position(|other| {
                    other.directory == *path
                        || context.workspace.root.join(&other.directory)
                            == std::path::Path::new(path)
                }) else {
                    continue;
                };
                let source_root = Self::find_root(&parents, position);
                let target_root = Self::find_root(&parents, target);
                parents[source_root.max(target_root)] = source_root.min(target_root);
            }
        }
        (0..members.len())
            .map(|position| Self::find_root(&parents, position))
            .collect()
    }

    fn find_product(context: &RuleContext, roots: &[usize], crate_name: &str) -> String {
        let normalized = crate_name.replace('-', "_");
        context
            .workspace
            .members
            .iter()
            .position(|member| member.name.replace('-', "_") == normalized)
            .map_or_else(
                || format!("crate:{normalized}"),
                |position| format!("product:{}", roots[position]),
            )
    }

    fn check_scope_declarations(context: &RuleContext) -> Vec<RuleFinding> {
        let roots = Self::list_product_roots(context);
        let mut seen = std::collections::BTreeSet::new();
        let mut products: std::collections::BTreeMap<String, Vec<ScopeDeclaration>> =
            std::collections::BTreeMap::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let is_scope = matches!(item.syntax, Item::Struct(_)) && item.name == "TaskScope";
                let is_src = index.modules[item.module].tree == SourceTree::Src;
                let key = format!(
                    "{}|{}|{}",
                    index.crate_name, item.file, item.span.start.line
                );
                if is_scope && is_src && seen.insert(key) {
                    products
                        .entry(Self::find_product(context, &roots, &index.crate_name))
                        .or_default()
                        .push(ScopeDeclaration {
                            ordinal,
                            index,
                            item,
                        });
                }
            }
        }
        products
            .values()
            .flat_map(|declarations| {
                let coexisting: Vec<&ScopeDeclaration> = declarations
                    .iter()
                    .filter(|declaration| {
                        declarations.iter().any(|other| {
                            !std::ptr::eq(*declaration, other)
                                && declaration
                                    .item
                                    .cfg
                                    .in_scope(declaration.ordinal)
                                    .and(&other.item.cfg.in_scope(other.ordinal))
                                    .is_satisfiable()
                        })
                    })
                    .collect();
                let message = format!(
                    "`TaskScope` is declared once in one product (crates linked by path dependencies); this is one of {} declarations (async.rs.md, Tasks)",
                    coexisting.len()
                );
                coexisting.into_iter().map(move |declaration| {
                    build_shape_item_finding(
                        declaration.index,
                        declaration.item,
                        "DiModuleLifecycle",
                        &message,
                    )
                })
            })
            .collect()
    }

    fn list_lifecycle_fields(
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
                let has_lifecycle = context
                    .resolve_workspace_type(ordinal, item.module, &field.ty)
                    .iter()
                    .any(|target| {
                        target.path.item.is_some_and(|identity| {
                            lifecycle_implementation(context, target.index, identity)
                        })
                    });
                has_lifecycle.then(|| name.to_string())
            })
            .collect()
    }

    fn is_active_method(
        implementation: &crate::source::model::IndexedImpl,
        method: &syn::ImplItemFn,
    ) -> bool {
        implementation
            .cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                &method.attrs,
            ))
            .is_satisfiable()
    }

    fn methods(context: &RuleContext, ordinal: usize, identity: usize, facts: &mut LifecycleFacts) {
        let index = &context.indexes[ordinal];
        let shared = Self::list_shared_fields(context, ordinal, identity);
        let task_scopes = Self::list_task_scope_fields(context, ordinal, identity);
        let lifecycle_children = Self::list_lifecycle_fields(context, ordinal, identity);
        let runtime_handles = if Self::is_technology_module(context, ordinal, identity) {
            Self::list_runtime_handle_fields(context, ordinal, identity)
        } else {
            Vec::new()
        };
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            for method in &implementation.syntax.items {
                let ImplItem::Fn(method) = method else {
                    continue;
                };
                if !Self::is_active_method(implementation, method) {
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
                        runtime_handles: &runtime_handles,
                    },
                );
                facts.invalid |= body.invalid;
                facts.invalid |= facts.owned.iter().any(|child| {
                    lifecycle_children.contains(child) && !body.forwarded.contains(child)
                });
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

    const BUILT_SCOPE_MESSAGE: &'static str = "A type that is not a DI module holds the scope it builds by value, not as an `Arc<TaskScope>` (async.rs.md, Tasks)";

    const HANDED_SCOPE_MESSAGE: &'static str = "A type that was handed a scope only spawns onto it: `abort` and `reload` belong to the scope's owner (async.rs.md, Tasks)";
}

impl Rule for DiModuleLifecycle {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<RuleFinding> = context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                (0..index.items.len()).flat_map(move |identity| {
                    Self::check_item(context, ordinal, identity)
                        .into_iter()
                        .chain(Self::check_handed_scope(context, ordinal, identity))
                })
            })
            .collect();
        findings.extend(Self::check_scope_declarations(context));
        findings
    }
}

struct ScopeShape {
    is_di: bool,
    is_by_value: bool,
    is_exclusive_pointer: bool,
}

struct ScopeDeclaration<'index> {
    ordinal: usize,
    index: &'index CrateIndex,
    item: &'index IndexedItem,
}
