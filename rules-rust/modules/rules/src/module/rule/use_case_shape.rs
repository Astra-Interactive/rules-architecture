use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::{build_shape_item_finding, find_kind_name, resolve_shape_type};
use crate::source::model::{CfgPredicate, CrateIndex, IndexedImpl, Resolution, SourceTree};
use crate::syntax::util::to_snake_case;
use std::collections::BTreeSet;
use syn::visit::Visit;
use syn::{ImplItem, Item, Visibility};

pub(crate) struct UseCaseShape;

impl UseCaseShape {
    fn read_block_value(block: &syn::Block) -> Option<&syn::Expr> {
        match block.stmts.as_slice() {
            [syn::Stmt::Expr(value, None)] => Some(value),
            _ => None,
        }
    }

    fn direct_argument(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(_) => true,
            syn::Expr::Reference(reference) => Self::direct_argument(&reference.expr),
            syn::Expr::Paren(paren) => Self::direct_argument(&paren.expr),
            _ => false,
        }
    }

    fn find_wrapped_value(call: &syn::ExprCall) -> Option<&syn::Expr> {
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return None;
        };
        let is_wrapper = path.path.is_ident("Ok") || path.path.is_ident("Some");
        match call.args.iter().collect::<Vec<_>>().as_slice() {
            [value] if is_wrapper => Some(value),
            _ => None,
        }
    }

    fn delegated_call(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Return(returned) => {
                returned.expr.as_deref().is_some_and(Self::delegated_call)
            }
            syn::Expr::Await(awaited) => Self::delegated_call(&awaited.base),
            syn::Expr::Try(attempt) => Self::delegated_call(&attempt.expr),
            syn::Expr::Call(call) if Self::find_wrapped_value(call).is_some() => {
                Self::find_wrapped_value(call).is_some_and(Self::delegated_call)
            }
            syn::Expr::Paren(paren) => Self::delegated_call(&paren.expr),
            syn::Expr::Block(block) if block.label.is_none() => {
                Self::read_block_value(&block.block).is_some_and(Self::delegated_call)
            }
            syn::Expr::MethodCall(_) | syn::Expr::Call(_) => true,
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
    fn find_receiver_field(expression: &syn::Expr) -> Option<&syn::ExprField> {
        match expression {
            syn::Expr::Field(field) if matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")) => {
                Some(field)
            }
            syn::Expr::Reference(reference) => Self::find_receiver_field(&reference.expr),
            syn::Expr::Paren(paren) => Self::find_receiver_field(&paren.expr),
            syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Deref(_)) => {
                Self::find_receiver_field(&unary.expr)
            }
            _ => None,
        }
    }

    fn is_contract_method(
        context: &RuleContext,
        target: &crate::source::model::WorkspaceResolvedPath,
        name: &syn::Ident,
    ) -> bool {
        let Some(identity) = target.path.item else {
            return false;
        };
        let item = &context.indexes[target.index].items[identity];
        let Item::Trait(contract) = &item.syntax else {
            return false;
        };
        find_kind_name(context, target.index, item.module) == "api"
            && contract.items.iter().any(|member| matches!(member, syn::TraitItem::Fn(method) if method.sig.ident == *name && method.sig.receiver().is_some()))
    }

    fn is_ufcs_contract_call(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        owner: &crate::source::model::IndexedItem,
        call: &syn::ExprCall,
    ) -> bool {
        let syn::Expr::Path(function) = call.func.as_ref() else {
            return false;
        };
        let mut path = function.path.clone();
        let Some(method) = path.segments.pop() else {
            return false;
        };
        let Some(receiver) = call.args.first().and_then(Self::find_receiver_field) else {
            return false;
        };
        let Item::Struct(structure) = &owner.syntax else {
            return false;
        };
        let field = match &receiver.member {
            syn::Member::Named(name) => structure
                .fields
                .iter()
                .find(|field| field.ident.as_ref() == Some(name)),
            syn::Member::Unnamed(position) => structure.fields.iter().nth(position.index as usize),
        };
        let Some(field) = field else {
            return false;
        };
        let receivers = context.resolve_workspace_type(ordinal, owner.module, &field.ty);
        context
            .resolve_workspace_path(ordinal, module, &path)
            .iter()
            .any(|target| {
                Self::is_contract_method(context, target, &method.ident)
                    && receivers.iter().any(|receiver| {
                        receiver.index == target.index
                            && receiver.path.item == target.path.item
                            && owner
                                .cfg
                                .in_scope(ordinal)
                                .and(&target.path.cfg.in_scope(target.index))
                                .and(&receiver.path.cfg.in_scope(receiver.index))
                                .is_satisfiable()
                    })
            })
    }

    fn is_forwarded_expression(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        owner: &crate::source::model::IndexedItem,
        expression: &syn::Expr,
        bindings: &std::collections::BTreeMap<String, bool>,
    ) -> bool {
        match expression {
            syn::Expr::Return(returned) => returned.expr.as_deref().is_some_and(|value| {
                Self::is_forwarded_expression(context, ordinal, module, owner, value, bindings)
            }),
            syn::Expr::Await(awaited) => Self::is_forwarded_expression(
                context,
                ordinal,
                module,
                owner,
                &awaited.base,
                bindings,
            ),
            syn::Expr::Paren(paren) => Self::is_forwarded_expression(
                context,
                ordinal,
                module,
                owner,
                &paren.expr,
                bindings,
            ),
            syn::Expr::Block(block) if block.label.is_none() => {
                Self::read_block_value(&block.block).is_some_and(|value| {
                    Self::is_forwarded_expression(context, ordinal, module, owner, value, bindings)
                })
            }
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| bindings.get(&name.to_string()) == Some(&true)),
            syn::Expr::Try(attempt) => Self::is_forwarded_expression(
                context,
                ordinal,
                module,
                owner,
                &attempt.expr,
                bindings,
            ),
            syn::Expr::Call(call) if Self::find_wrapped_value(call).is_some() => {
                Self::find_wrapped_value(call).is_some_and(|value| {
                    Self::is_forwarded_expression(context, ordinal, module, owner, value, bindings)
                })
            }
            syn::Expr::MethodCall(call) => {
                call.args.iter().all(Self::direct_argument)
                    && matches!(call.receiver.as_ref(), syn::Expr::Field(field) if matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")))
            }
            syn::Expr::Call(call) => {
                Self::is_ufcs_contract_call(context, ordinal, module, owner, call)
                    && call.args.iter().skip(1).all(Self::direct_argument)
            }
            _ => false,
        }
    }

    fn is_passthrough(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        owner: &crate::source::model::IndexedItem,
        block: &syn::Block,
    ) -> bool {
        let mut bindings = std::collections::BTreeMap::new();
        let mut calls = 0;
        for (position, statement) in block.stmts.iter().enumerate() {
            match statement {
                syn::Stmt::Local(local) => {
                    let pattern = match &local.pat {
                        syn::Pat::Type(typed) => typed.pat.as_ref(),
                        other => other,
                    };
                    let syn::Pat::Ident(binding) = pattern else {
                        return false;
                    };
                    if binding.mutability.is_some() {
                        return false;
                    }
                    let Some(initializer) = &local.init else {
                        return false;
                    };
                    let forwarded = Self::is_forwarded_expression(
                        context,
                        ordinal,
                        module,
                        owner,
                        &initializer.expr,
                        &bindings,
                    );
                    if !forwarded {
                        return false;
                    }
                    calls += usize::from(Self::delegated_call(&initializer.expr));
                    bindings.insert(binding.ident.to_string(), true);
                }
                syn::Stmt::Expr(expression, _) if position + 1 == block.stmts.len() => {
                    calls += usize::from(Self::delegated_call(expression));
                    return calls == 1
                        && Self::is_forwarded_expression(
                            context, ordinal, module, owner, expression, &bindings,
                        );
                }
                _ => return false,
            }
        }
        false
    }

    fn method_is_invalid(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        owner: &crate::source::model::IndexedItem,
        method: &syn::ImplItemFn,
    ) -> bool {
        let index = &context.indexes[ordinal];
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
        let passthrough = method.sig.ident == "invoke"
            && Self::is_passthrough(context, ordinal, module, owner, &method.block);
        (method.sig.ident == "invoke" && method.sig.asyncness.is_some() && !visitor.has_await)
            || visitor.invokes_usecase
            || passthrough
    }

    fn methods_are_invalid(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> bool {
        let index = &context.indexes[ordinal];
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
                let active = item
                    .cfg
                    .and(&implementation.cfg)
                    .and(&CfgPredicate::from_attributes(&method.attrs))
                    .is_satisfiable();
                if !active {
                    continue;
                }
                let public = !matches!(method.vis, Visibility::Inherited)
                    || implementation.syntax.trait_.is_some();
                if public && method.sig.ident != "new" && method.sig.ident != "invoke" {
                    return true;
                }
                if Self::method_is_invalid(context, ordinal, implementation.module, item, method) {
                    return true;
                }
                has_invoke |= public && method.sig.ident == "invoke";
            }
        }
        !has_invoke
    }

    fn read_owned_implementations<'index>(
        index: &'index CrateIndex,
        item: &crate::source::model::IndexedItem,
    ) -> Vec<&'index IndexedImpl> {
        let Some(identity) = index
            .items
            .iter()
            .position(|candidate| std::ptr::eq(candidate, item))
        else {
            return Vec::new();
        };
        index
            .implementations
            .iter()
            .filter(|implementation| {
                crate::module::util::owns_implementation(index, identity, implementation)
            })
            .collect()
    }

    fn is_contract_trait(
        context: &RuleContext,
        target: &crate::source::model::WorkspaceResolvedPath,
    ) -> bool {
        target.path.item.is_some_and(|identity| {
            let item = &context.indexes[target.index].items[identity];
            matches!(item.syntax, Item::Trait(_))
                && find_kind_name(context, target.index, item.module) == "api"
        })
    }

    fn read_type_predicate(predicate: &syn::WherePredicate) -> Option<&syn::PredicateType> {
        match predicate {
            syn::WherePredicate::Type(typed) => Some(typed),
            _ => None,
        }
    }

    fn collect_generics_bounds<'syntax>(
        generics: &'syntax syn::Generics,
        name: &syn::Ident,
    ) -> Vec<&'syntax syn::TypeParamBound> {
        let declared = generics
            .type_params()
            .filter(|parameter| parameter.ident == *name)
            .flat_map(|parameter| parameter.bounds.iter());
        let predicated = generics
            .where_clause
            .iter()
            .flat_map(|clause| clause.predicates.iter())
            .filter_map(Self::read_type_predicate)
            .filter(|predicate| {
                matches!(&predicate.bounded_ty, syn::Type::Path(bounded) if bounded.path.is_ident(name))
            })
            .flat_map(|predicate| predicate.bounds.iter());
        declared.chain(predicated).collect()
    }

    fn collect_parameter_bounds<'syntax>(
        typed: &'syntax syn::Type,
        method: &'syntax syn::ImplItemFn,
        implementation: &'syntax syn::ItemImpl,
    ) -> Vec<&'syntax syn::TypeParamBound> {
        match typed {
            syn::Type::Reference(reference) => {
                Self::collect_parameter_bounds(&reference.elem, method, implementation)
            }
            syn::Type::Paren(paren) => {
                Self::collect_parameter_bounds(&paren.elem, method, implementation)
            }
            syn::Type::ImplTrait(opaque) => opaque.bounds.iter().collect(),
            syn::Type::Path(path) if path.qself.is_none() => {
                let Some(name) = path.path.get_ident() else {
                    return Vec::new();
                };
                [&method.sig.generics, &implementation.generics]
                    .into_iter()
                    .flat_map(|generics| Self::collect_generics_bounds(generics, name))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn generic_contract_parameter_invalid(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
        method: &syn::ImplItemFn,
        typed: &syn::Type,
    ) -> bool {
        Self::collect_parameter_bounds(typed, method, &implementation.syntax)
            .iter()
            .any(|bound| {
                let syn::TypeParamBound::Trait(bound) = bound else {
                    return false;
                };
                context
                    .resolve_workspace_path(ordinal, implementation.module, &bound.path)
                    .iter()
                    .any(|target| Self::is_contract_trait(context, target))
            })
    }

    fn has_contract_field(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        structure: &syn::ItemStruct,
    ) -> bool {
        structure.fields.iter().any(|field| {
            let cfg = item.cfg.and(&CfgPredicate::from_attributes(&field.attrs));
            cfg.is_satisfiable()
                && context
                    .resolve_workspace_type(ordinal, item.module, &field.ty)
                    .iter()
                    .any(|target| Self::is_contract_trait(context, target))
        })
    }

    fn is_constructor(member: &ImplItem) -> bool {
        matches!(member, ImplItem::Fn(method) if method.sig.ident == "new")
    }

    fn has_constructor(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> bool {
        Self::read_owned_implementations(&context.indexes[ordinal], item)
            .iter()
            .any(|implementation| implementation.syntax.items.iter().any(Self::is_constructor))
    }

    fn is_constructor_parameter_invalid(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
        method: &syn::ImplItemFn,
        parameter: &syn::FnArg,
    ) -> bool {
        let syn::FnArg::Typed(parameter) = parameter else {
            return false;
        };
        Self::contract_parameter_invalid(context, ordinal, implementation.module, &parameter.ty)
            || Self::generic_contract_parameter_invalid(
                context,
                ordinal,
                implementation,
                method,
                &parameter.ty,
            )
    }

    fn is_constructor_invalid(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
        member: &ImplItem,
    ) -> bool {
        let ImplItem::Fn(method) = member else {
            return false;
        };
        method.sig.ident == "new"
            && method.sig.inputs.iter().any(|parameter| {
                Self::is_constructor_parameter_invalid(
                    context,
                    ordinal,
                    implementation,
                    method,
                    parameter,
                )
            })
    }

    fn constructor_contracts_invalid(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> bool {
        let index = &context.indexes[ordinal];
        Self::read_owned_implementations(index, item)
            .iter()
            .any(|implementation| {
                implementation.syntax.items.iter().any(|member| {
                    Self::is_constructor_invalid(context, ordinal, implementation, member)
                })
            })
    }

    fn build_finding(index: &CrateIndex, item: &crate::source::model::IndexedItem) -> RuleFinding {
        build_shape_item_finding(
            index,
            item,
            "UseCaseShape",
            "a VerbObjectUseCase lives in usecase, exposes only new/invoke, invokes no other use case and awaits in async invoke",
        )
    }

    fn check_free_function(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let in_kind = find_kind_name(context, ordinal, item.module) == "usecase";
        let production = index.modules[item.module].tree == SourceTree::Src;
        (in_kind && production && item.cfg.is_satisfiable())
            .then(|| Self::build_finding(index, item))
    }

    fn is_concrete_field(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        structure: &syn::ItemStruct,
        field: &syn::Field,
    ) -> bool {
        let cfg = item.cfg.and(&CfgPredicate::from_attributes(&field.attrs));
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
    }

    fn lacks_constructor_for_contracts(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        structure: &syn::ItemStruct,
    ) -> bool {
        let production = context.indexes[ordinal].modules[item.module].tree == SourceTree::Src;
        production
            && Self::has_contract_field(context, ordinal, item, structure)
            && !Self::has_constructor(context, ordinal, item)
    }

    fn check_item(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> Option<RuleFinding> {
        if matches!(item.syntax, Item::Fn(_)) {
            return Self::check_free_function(context, ordinal, item);
        }
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
        let concrete = structure
            .fields
            .iter()
            .any(|field| Self::is_concrete_field(context, ordinal, item, structure, field));
        let invalid = !in_kind
            || !Self::has_two_words(&item.name)
            || fields
            || concrete
            || Self::lacks_constructor_for_contracts(context, ordinal, item, structure)
            || Self::constructor_contracts_invalid(context, ordinal, item)
            || Self::methods_are_invalid(context, ordinal, item);
        invalid.then(|| Self::build_finding(index, item))
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
    fn bind_closure_input(&mut self, input: &syn::Pat) {
        let (binding, typed) = match input {
            syn::Pat::Ident(binding) => (Some(binding), false),
            syn::Pat::Type(typed) => match typed.pat.as_ref() {
                syn::Pat::Ident(binding) => (
                    Some(binding),
                    UseCaseShape::mentions_use_case(self.index, self.module, &typed.ty),
                ),
                _ => (None, false),
            },
            _ => (None, false),
        };
        let Some(binding) = binding else {
            return;
        };
        self.local_usecases.remove(&binding.ident.to_string());
        if typed {
            self.local_usecases.insert(binding.ident.to_string());
        }
    }

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
    fn visit_expr_await(&mut self, awaited: &'ast syn::ExprAwait) {
        self.has_await = true;
        syn::visit::visit_expr_await(self, awaited);
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let bindings = self.local_usecases.clone();
        let has_await = self.has_await;
        for input in &closure.inputs {
            self.bind_closure_input(input);
        }
        self.visit_expr(&closure.body);
        self.local_usecases = bindings;
        self.has_await = has_await;
    }
    fn visit_expr_async(&mut self, block: &'ast syn::ExprAsync) {
        let has_await = self.has_await;
        syn::visit::visit_expr_async(self, block);
        self.has_await = has_await;
    }
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
