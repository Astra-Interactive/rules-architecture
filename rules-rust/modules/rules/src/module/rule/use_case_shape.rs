use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, resolve_shape_type};
use crate::source::model::{CrateIndex, Resolution};
use crate::syntax::util::to_snake_case;
use std::collections::BTreeSet;
use syn::visit::Visit;
use syn::{ImplItem, Item, Visibility};

pub(crate) struct UseCaseShape;

impl UseCaseShape {
    fn direct_argument(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(_) => true,
            syn::Expr::Reference(reference) => Self::direct_argument(&reference.expr),
            syn::Expr::Paren(paren) => Self::direct_argument(&paren.expr),
            _ => false,
        }
    }

    fn delegated_call(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Return(returned) => {
                returned.expr.as_deref().is_some_and(Self::delegated_call)
            }
            syn::Expr::Await(awaited) => Self::delegated_call(&awaited.base),
            syn::Expr::Paren(paren) => Self::delegated_call(&paren.expr),
            syn::Expr::MethodCall(_) => true,
            _ => false,
        }
    }

    fn contract_parameter_invalid(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
    ) -> bool {
        let contract = context
            .resolve_workspace_type(ordinal, module, typed)
            .iter()
            .any(|target| {
                target.path.item.is_some_and(|identity| {
                    let item = &context.indexes[target.index].items[identity];
                    matches!(item.syntax, Item::Trait(_))
                        && find_kind_name(context, target.index, item.module) == "api"
                })
            });
        contract && !context.resolve_type_syntax(ordinal, module, typed).iter().any(|resolved| {
            let syn::Type::Path(path) = &resolved.typed else { return false; };
            let arc = context.resolve_outer_external_type_paths(ordinal, module, &resolved.typed).iter().any(|path| ["std", "alloc"].contains(&path.package.as_str()) && path.segments.last().is_some_and(|name| name == "Arc"));
            arc && path.path.segments.last().is_some_and(|segment| matches!(&segment.arguments, syn::PathArguments::AngleBracketed(arguments) if arguments.args.iter().any(|argument| matches!(argument, syn::GenericArgument::Type(syn::Type::TraitObject(_))))))
        })
    }

    fn has_two_words(name: &str) -> bool {
        name.strip_suffix("UseCase").is_some_and(|stem| {
            to_snake_case(stem)
                .split('_')
                .filter(|word| !word.is_empty())
                .count()
                >= 2
        })
    }

    fn mentions_use_case(index: &CrateIndex, module: usize, ty: &syn::Type) -> bool {
        let mut visitor = UseCaseTypeVisitor {
            index,
            module,
            found: false,
        };
        visitor.visit_type(ty);
        visitor.found
    }
}

impl UseCaseShape {
    fn forwarded_expression(
        expression: &syn::Expr,
        bindings: &std::collections::BTreeMap<String, bool>,
    ) -> bool {
        match expression {
            syn::Expr::Return(returned) => returned
                .expr
                .as_deref()
                .is_some_and(|value| Self::forwarded_expression(value, bindings)),
            syn::Expr::Await(awaited) => Self::forwarded_expression(&awaited.base, bindings),
            syn::Expr::Paren(paren) => Self::forwarded_expression(&paren.expr, bindings),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| bindings.get(&name.to_string()) == Some(&true)),
            syn::Expr::MethodCall(call) => {
                call.args.iter().all(Self::direct_argument)
                    && matches!(call.receiver.as_ref(), syn::Expr::Field(field) if matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")))
            }
            _ => false,
        }
    }

    fn is_passthrough(block: &syn::Block) -> bool {
        let mut bindings = std::collections::BTreeMap::new();
        let mut calls = 0;
        for (position, statement) in block.stmts.iter().enumerate() {
            match statement {
                syn::Stmt::Local(local) => {
                    let syn::Pat::Ident(binding) = &local.pat else {
                        return false;
                    };
                    if binding.mutability.is_some() {
                        return false;
                    }
                    let Some(initializer) = &local.init else {
                        return false;
                    };
                    if !Self::forwarded_expression(&initializer.expr, &bindings) {
                        return false;
                    }
                    calls += usize::from(Self::delegated_call(&initializer.expr));
                    bindings.insert(binding.ident.to_string(), true);
                }
                syn::Stmt::Expr(expression, _) if position + 1 == block.stmts.len() => {
                    calls += usize::from(Self::delegated_call(expression));
                    return calls == 1 && Self::forwarded_expression(expression, &bindings);
                }
                _ => return false,
            }
        }
        false
    }

    fn method_is_invalid(index: &CrateIndex, module: usize, method: &syn::ImplItemFn) -> bool {
        let mut visitor = InvokeAwaitVisitor {
            index,
            module,
            local_usecases: BTreeSet::new(),
            invokes_usecase: false,
            has_await: false,
        };
        for parameter in &method.sig.inputs {
            let syn::FnArg::Typed(parameter) = parameter else {
                continue;
            };
            if !Self::mentions_use_case(index, module, &parameter.ty) {
                continue;
            }
            if let syn::Pat::Ident(binding) = parameter.pat.as_ref() {
                visitor.local_usecases.insert(binding.ident.to_string());
            }
        }
        visitor.visit_block(&method.block);
        let passthrough = method.sig.ident == "invoke" && Self::is_passthrough(&method.block);
        (method.sig.ident == "invoke" && method.sig.asyncness.is_some() && !visitor.has_await)
            || visitor.invokes_usecase
            || passthrough
    }

    fn methods_are_invalid(index: &CrateIndex, item: &crate::source::model::IndexedItem) -> bool {
        let mut has_invoke = false;
        for implementation in &index.implementations {
            let targets =
                resolve_shape_type(index, implementation.module, &implementation.syntax.self_ty);
            if !targets
                .iter()
                .any(|target| std::ptr::eq(&index.items[*target], item))
            {
                continue;
            }
            if !item.cfg.and(&implementation.cfg).is_satisfiable() {
                continue;
            }
            for method in &implementation.syntax.items {
                let ImplItem::Fn(method) = method else {
                    continue;
                };
                let public = !matches!(method.vis, Visibility::Inherited)
                    || implementation.syntax.trait_.is_some();
                if public && method.sig.ident != "new" && method.sig.ident != "invoke" {
                    return true;
                }
                if Self::method_is_invalid(index, implementation.module, method) {
                    return true;
                }
                has_invoke |= public && method.sig.ident == "invoke";
            }
        }
        !has_invoke
    }

    fn constructor_contracts_invalid(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let Some(identity) = index
            .items
            .iter()
            .position(|candidate| std::ptr::eq(candidate, item))
        else {
            return false;
        };
        for implementation in index.implementations.iter().filter(|implementation| {
            crate::module::util::owns_implementation(index, identity, implementation)
        }) {
            for member in &implementation.syntax.items {
                let ImplItem::Fn(method) = member else {
                    continue;
                };
                if method.sig.ident != "new" {
                    continue;
                }
                if method.sig.inputs.iter().any(|parameter| match parameter {
                    syn::FnArg::Typed(parameter) => Self::contract_parameter_invalid(
                        context,
                        ordinal,
                        implementation.module,
                        &parameter.ty,
                    ),
                    _ => false,
                }) {
                    return true;
                }
            }
        }
        false
    }

    fn check_item(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> Option<RuleFinding> {
        let Item::Struct(structure) = &item.syntax else {
            return None;
        };
        let index = &context.indexes[ordinal];
        let in_kind = find_kind_name(context, ordinal, item.module) == "usecase";
        if !in_kind && !item.name.ends_with("UseCase") {
            return None;
        }
        let fields = structure
            .fields
            .iter()
            .any(|field| Self::mentions_use_case(index, item.module, &field.ty));
        let concrete = structure.fields.iter().any(|field| {
            let cfg = item
                .cfg
                .and(&crate::source::model::CfgPredicate::from_attributes(
                    &field.attrs,
                ));
            if !cfg.is_satisfiable() {
                return false;
            }
            let generic = crate::module::rule::PlacementTypeVisitor::collect_paths(&field.ty)
                .iter()
                .any(|path| {
                    path.get_ident().is_some_and(|name| {
                        structure
                            .generics
                            .type_params()
                            .any(|parameter| parameter.ident == *name)
                    })
                });
            let implementation = context
                .resolve_workspace_type(ordinal, item.module, &field.ty)
                .iter()
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        matches!(
                            context.indexes[target.index].items[identity].syntax,
                            syn::Item::Struct(_)
                        ) && crate::module::util::workspace_implementation_under(
                            context,
                            target.index,
                            identity,
                            &cfg.and(&target.path.cfg),
                        )
                    })
                });
            generic || implementation
        });
        let invalid = !in_kind
            || !Self::has_two_words(&item.name)
            || fields
            || concrete
            || Self::constructor_contracts_invalid(context, ordinal, item)
            || Self::methods_are_invalid(index, item);
        invalid.then(|| build_shape_item_finding(index, item, "UseCaseShape", "a VerbObjectUseCase lives in usecase, exposes only new/invoke, invokes no other use case and awaits in async invoke"))
    }
}

impl Rule for UseCaseShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .items
                    .iter()
                    .filter_map(move |item| Self::check_item(context, ordinal, item))
            })
            .collect()
    }
}

struct UseCaseTypeVisitor<'index> {
    index: &'index CrateIndex,
    module: usize,
    found: bool,
}

impl<'ast> Visit<'ast> for UseCaseTypeVisitor<'_> {
    fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
        self.found |= ty
            .path
            .segments
            .iter()
            .any(|segment| segment.ident.to_string().ends_with("UseCase"));
        self.found |= self
            .index
            .resolve_type_items(self.module, &syn::Type::Path(ty.clone()))
            .iter()
            .any(|target| self.index.items[*target].name.ends_with("UseCase"));
        syn::visit::visit_type_path(self, ty);
    }
}

struct InvokeAwaitVisitor<'index> {
    index: &'index CrateIndex,
    module: usize,
    local_usecases: BTreeSet<String>,
    invokes_usecase: bool,
    has_await: bool,
}

impl InvokeAwaitVisitor<'_> {
    fn is_usecase(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => {
                if path
                    .path
                    .get_ident()
                    .is_some_and(|name| self.local_usecases.contains(&name.to_string()))
                {
                    return true;
                }
                match self.index.resolve_path(self.module, &path.path) {
                    Resolution::Internal(paths) => paths
                        .iter()
                        .filter_map(|path| path.item)
                        .any(|target| self.index.items[target].name.ends_with("UseCase")),
                    _ => false,
                }
            }
            syn::Expr::Call(call) => self.is_usecase(&call.func),
            syn::Expr::Paren(paren) => self.is_usecase(&paren.expr),
            syn::Expr::Reference(reference) => self.is_usecase(&reference.expr),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for InvokeAwaitVisitor<'_> {
    fn visit_expr_await(&mut self, _: &'ast syn::ExprAwait) {
        self.has_await = true;
    }
    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.invokes_usecase |= call.method == "invoke" && self.is_usecase(&call.receiver);
        syn::visit::visit_expr_method_call(self, call);
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let (binding, typed) = match &local.pat {
            syn::Pat::Ident(binding) => (Some(binding), false),
            syn::Pat::Type(typed) => (
                if let syn::Pat::Ident(binding) = typed.pat.as_ref() {
                    Some(binding)
                } else {
                    None
                },
                UseCaseShape::mentions_use_case(self.index, self.module, &typed.ty),
            ),
            _ => (None, false),
        };
        if let Some(binding) = binding {
            let usecase = typed
                || local
                    .init
                    .as_ref()
                    .is_some_and(|initializer| self.is_usecase(&initializer.expr));
            self.local_usecases.remove(&binding.ident.to_string());
            if usecase {
                self.local_usecases.insert(binding.ident.to_string());
            }
        }
        syn::visit::visit_local(self, local);
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let bindings = self.local_usecases.clone();
        syn::visit::visit_block(self, block);
        self.local_usecases = bindings;
    }
}
