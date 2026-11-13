use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::is_di_type;
use crate::module::util::{build_shape_item_finding, find_kind_name};
use syn::visit::Visit;

pub(crate) struct DiModuleShape;

impl DiModuleShape {
    fn provider_has_contract(
        context: &RuleContext,
        provider: usize,
        identity: usize,
        contract: &crate::source::model::WorkspaceResolvedPath,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let item = &context.indexes[provider].items[identity];
        if !is_di_type(context, provider, item) {
            return false;
        }
        let syn::Item::Struct(structure) = &item.syntax else {
            return false;
        };
        if !cfg.and(&item.cfg.in_scope(provider)).is_satisfiable() {
            return false;
        }
        structure.fields.iter().any(|field| {
            let compatible = cfg.and(&item.cfg.in_scope(provider)).and(
                &crate::source::model::CfgPredicate::from_attributes(&field.attrs)
                    .in_scope(provider),
            );
            if !compatible.is_satisfiable() {
                return false;
            }
            context
                .resolve_workspace_type(provider, item.module, &field.ty)
                .iter()
                .any(|field| {
                    contract.index == field.index
                        && contract.path.item == field.path.item
                        && compatible
                            .and(&field.path.cfg.in_scope(field.index))
                            .is_satisfiable()
                })
        })
    }

    fn available_provider(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        contract: &crate::source::model::WorkspaceResolvedPath,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let is_contract = contract.path.item.is_some_and(|identity| {
            matches!(
                context.indexes[contract.index].items[identity].syntax,
                syn::Item::Trait(_)
            )
        });
        if !is_contract {
            return false;
        }
        for (provider, source) in context.indexes.iter().enumerate() {
            let available = provider == ordinal
                || context
                    .workspace
                    .members
                    .iter()
                    .find(|member| {
                        member.name.replace('-', "_") == context.indexes[ordinal].crate_name
                    })
                    .is_some_and(|member| {
                        member.dependencies.iter().any(|dependency| {
                            dependency.kind == crate::workspace::model::DependencyKind::Normal
                                && dependency.name.replace('-', "_") == source.crate_name
                        })
                    });
            if !available {
                continue;
            }
            if source
                .items
                .iter()
                .enumerate()
                .any(|(provider_identity, _)| {
                    (provider != ordinal || provider_identity != identity)
                        && Self::provider_has_contract(
                            context,
                            provider,
                            provider_identity,
                            contract,
                            cfg,
                        )
                })
            {
                return true;
            }
        }
        false
    }

    fn parameter_has_provider(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        module: usize,
        parameter: &syn::FnArg,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let syn::FnArg::Typed(parameter) = parameter else {
            return false;
        };
        context
            .resolve_workspace_type(ordinal, module, &parameter.ty)
            .iter()
            .any(|contract| Self::available_provider(context, ordinal, identity, contract, cfg))
    }

    fn individual_service_parameters(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
    ) -> bool {
        let index = &context.indexes[ordinal];
        index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .any(|implementation| {
                implementation.syntax.items.iter().any(|member| {
                    let syn::ImplItem::Fn(method) = member else {
                        return false;
                    };
                    if method.sig.ident != "new" {
                        return false;
                    }
                    let cfg = index.items[identity]
                        .cfg
                        .and(&implementation.cfg)
                        .and(&crate::source::model::CfgPredicate::from_attributes(
                            &method.attrs,
                        ))
                        .in_scope(ordinal);
                    method.sig.inputs.iter().any(|parameter| {
                        Self::parameter_has_provider(
                            context,
                            ordinal,
                            identity,
                            implementation.module,
                            parameter,
                            &cfg,
                        )
                    })
                })
            })
    }

    fn deferred_accessor(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        field: &str,
    ) -> bool {
        use syn::visit::Visit;
        let index = &context.indexes[ordinal];
        let methods: Vec<_> = index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .flat_map(|implementation| &implementation.syntax.items)
            .filter_map(|member| match member {
                syn::ImplItem::Fn(method) if method.sig.receiver().is_some() => Some(method),
                _ => None,
            })
            .collect();
        let mut initialized = std::collections::BTreeSet::new();
        for _ in 0..methods.len() {
            for method in &methods {
                let mut visitor = DeferredAccessorVisitor {
                    field,
                    initialized: &initialized,
                    found: false,
                };
                visitor.visit_block(&method.block);
                if visitor.found {
                    initialized.insert(method.sig.ident.to_string());
                }
            }
        }
        initialized.contains(field) || initialized.contains(&format!("provide_{field}"))
    }

    fn concrete_contract(
        context: &RuleContext,
        target: &crate::source::model::WorkspaceResolvedPath,
    ) -> bool {
        let Some(identity) = target.path.item else {
            return false;
        };
        let index = &context.indexes[target.index];
        if is_di_type(context, target.index, &index.items[identity]) {
            return false;
        }
        if !matches!(index.items[identity].syntax, syn::Item::Struct(_)) {
            return false;
        }
        index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .any(|implementation| {
                let Some((path, _)) = &implementation.syntax.trait_ else {
                    return false;
                };
                context
                    .resolve_workspace_path(target.index, implementation.module, path)
                    .iter()
                    .any(|contract| {
                        contract.path.item.is_some_and(|identity| {
                            find_kind_name(
                                context,
                                contract.index,
                                context.indexes[contract.index].items[identity].module,
                            ) == "api"
                        })
                    })
            })
    }

    fn invalid_deferred(
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
                path.package == "std" && path.segments.last().is_some_and(|name| name == "OnceLock")
            })
            && field.ident.as_ref().is_some_and(|name| {
                let identity = context.indexes[ordinal]
                    .items
                    .iter()
                    .position(|candidate| std::ptr::eq(candidate, item));
                identity.is_some_and(|identity| {
                    !Self::deferred_accessor(context, ordinal, identity, &name.to_string())
                })
            })
    }

    fn service_fields(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> bool {
        let syn::Item::Struct(structure) = &item.syntax else {
            return false;
        };
        structure.fields.iter().any(|field| {
            let cfg = item
                .cfg
                .and(&crate::source::model::CfgPredicate::from_attributes(
                    &field.attrs,
                ));
            if !cfg.is_satisfiable() {
                return false;
            }
            let deferred = context
                .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                .iter()
                .any(|path| {
                    ["std", "core", "tokio", "once_cell"].contains(&path.package.as_str())
                        && path.segments.last().is_some_and(|name| {
                            ["OnceCell", "LazyCell", "LazyLock", "Lazy"].contains(&name.as_str())
                        })
                });
            let concrete = context
                .resolve_workspace_type(ordinal, item.module, &field.ty)
                .iter()
                .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        Self::service_identity(context, target.index, identity)
                    })
                });
            let shared = context
                .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                .iter()
                .any(|path| {
                    ["std", "alloc"].contains(&path.package.as_str())
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["Arc", "OnceLock"].contains(&name.as_str()))
                });
            let concrete_contract = concrete
                && context
                    .resolve_workspace_type(ordinal, item.module, &field.ty)
                    .iter()
                    .any(|target| Self::concrete_contract(context, target));
            let invalid_deferred = Self::invalid_deferred(context, ordinal, item, field, &cfg);
            deferred
                || concrete_contract
                || invalid_deferred
                || (concrete
                    && !shared
                    && !context
                        .resolve_workspace_type(ordinal, item.module, &field.ty)
                        .iter()
                        .any(|target| crate::module::util::resolved_di_type(context, target)))
        })
    }

    fn service_identity(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let item = &context.indexes[ordinal].items[identity];
        item.name != "TaskScope"
            && (crate::module::util::workspace_implementation(context, ordinal, identity)
                || find_kind_name(context, ordinal, item.module) == "usecase"
                || (matches!(item.syntax, syn::Item::Trait(_))
                    && find_kind_name(context, ordinal, item.module) == "api"))
    }

    fn inside_deferred_init(
        implementation: &crate::source::model::IndexedImpl,
        span: &crate::source::model::NodeSpan,
    ) -> bool {
        let mut visitor = DeferredInitVisitor { spans: Vec::new() };
        visitor.visit_item_impl(&implementation.syntax);
        visitor.spans.iter().any(|init| init.contains(span))
    }

    fn constructs_outside_new(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        context.name_index().uses.iter().any(|usage| {
            let eligible = usage.source_index == ordinal
                && !usage.is_test
                && !usage.ambiguous
                && usage.function.as_deref().is_some_and(|name| name != "new")
                && crate::module::util::is_construction_use(context, usage)
                && crate::module::util::workspace_implementation(
                    context,
                    usage.target_index,
                    usage.target_item,
                );
            eligible
                && index.implementations.iter().any(|implementation| {
                    implementation.file == usage.file
                        && implementation.span.contains(&usage.span)
                        && !Self::inside_deferred_init(implementation, &usage.span)
                        && crate::module::util::owns_implementation(index, identity, implementation)
                })
        })
    }

    fn item_generics(item: &crate::source::model::IndexedItem) -> Option<&syn::Generics> {
        match &item.syntax {
            syn::Item::Struct(value) => Some(&value.generics),
            syn::Item::Enum(value) => Some(&value.generics),
            _ => None,
        }
    }
    fn has_contract_bound<'syntax>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        bounds: impl IntoIterator<Item = &'syntax syn::TypeParamBound>,
    ) -> bool {
        bounds.into_iter().any(|bound| {
            let syn::TypeParamBound::Trait(bound) = bound else {
                return false;
            };
            context
                .resolve_workspace_path(ordinal, module, &bound.path)
                .iter()
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        matches!(
                            context.indexes[target.index].items[identity].syntax,
                            syn::Item::Trait(_)
                        )
                    })
                })
        })
    }

    fn generic_contract(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        generics: &syn::Generics,
    ) -> bool {
        let mut bounds: Vec<_> = generics
            .type_params()
            .flat_map(|parameter| parameter.bounds.iter())
            .collect();
        if let Some(clause) = &generics.where_clause {
            for predicate in &clause.predicates {
                if let syn::WherePredicate::Type(predicate) = predicate {
                    bounds.extend(&predicate.bounds);
                }
            }
        }
        Self::has_contract_bound(context, ordinal, module, bounds)
    }

    fn is_contract_impl_trait_parameter(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        parameter: &syn::FnArg,
    ) -> bool {
        let syn::FnArg::Typed(parameter) = parameter else {
            return false;
        };
        let mut parameter_type = parameter.ty.as_ref();
        while let syn::Type::Reference(reference) = parameter_type {
            parameter_type = reference.elem.as_ref();
        }
        let syn::Type::ImplTrait(opaque) = parameter_type else {
            return false;
        };
        Self::has_contract_bound(context, ordinal, module, &opaque.bounds)
    }

    fn is_contract_bound_function(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        method: &syn::ImplItemFn,
    ) -> bool {
        Self::generic_contract(context, ordinal, module, &method.sig.generics)
            || method.sig.inputs.iter().any(|parameter| {
                Self::is_contract_impl_trait_parameter(context, ordinal, module, parameter)
            })
    }

    fn has_contract_bound_in_impl(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        reads_constructor: bool,
    ) -> bool {
        let index = &context.indexes[ordinal];
        index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .any(|implementation| {
                let module = implementation.module;
                Self::generic_contract(context, ordinal, module, &implementation.syntax.generics)
                    || (implementation.syntax.trait_.is_none()
                        && implementation
                            .syntax
                            .items
                            .iter()
                            .any(|member| match member {
                                syn::ImplItem::Fn(method) => {
                                    (reads_constructor || method.sig.ident != "new")
                                        && Self::is_contract_bound_function(
                                            context, ordinal, module, method,
                                        )
                                }
                                _ => false,
                            }))
            })
    }
}

struct DeferredAccessorVisitor<'scope> {
    field: &'scope str,
    initialized: &'scope std::collections::BTreeSet<String>,
    found: bool,
}

impl<'ast> syn::visit::Visit<'ast> for DeferredAccessorVisitor<'_> {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let own_field = matches!(call.receiver.as_ref(), syn::Expr::Field(field) if matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")) && matches!(&field.member, syn::Member::Named(name) if name == self.field));
        let delegated = matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self"))
            && self.initialized.contains(&call.method.to_string());
        self.found |= (own_field && call.method == "get_or_init") || delegated;
        syn::visit::visit_expr_method_call(self, call);
    }
}

struct DeferredInitVisitor {
    spans: Vec<crate::source::model::NodeSpan>,
}

impl<'ast> syn::visit::Visit<'ast> for DeferredInitVisitor {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let own_field = matches!(call.receiver.as_ref(), syn::Expr::Field(field) if matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")));
        if own_field && call.method == "get_or_init" {
            self.spans.push(crate::source::util::find_node_span(
                call.paren_token.span.join(),
            ));
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

impl Rule for DiModuleShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in index.items.iter().enumerate() {
                let di = is_di_type(context, ordinal, item);
                let shape =
                    di && !matches!(item.syntax, syn::Item::Struct(_) | syn::Item::Trait(_));
                let kind = find_kind_name(context, ordinal, item.module);
                let entry = context.is_entry_kind(&index.crate_name, kind);
                let relevant = di || kind == "usecase" || entry;
                let struct_bound = relevant
                    && Self::item_generics(item).is_some_and(|generics| {
                        Self::generic_contract(context, ordinal, item.module, generics)
                    });
                let impl_bound = relevant
                    && Self::has_contract_bound_in_impl(context, ordinal, identity, di || entry);
                let bound = struct_bound || impl_bound;
                let constructs_outside_new =
                    di && Self::constructs_outside_new(context, ordinal, identity);
                let service = di
                    && (Self::service_fields(context, ordinal, item)
                        || Self::individual_service_parameters(context, ordinal, identity)
                        || constructs_outside_new);
                let message = if constructs_outside_new {
                    "DI modules construct services/dependencies only in new (di-manual.rs.md)"
                } else {
                    "a DI module is a struct or a trait on a package-layout contract trigger; DI, use cases and entry-point types receive trait objects instead of workspace-contract generic parameters"
                };
                if shape || bound || service {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "DiModuleShape",
                        message,
                    ));
                }
            }
        }
        findings
    }
}
