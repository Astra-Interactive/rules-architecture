use crate::check::model::RuleContext;
use crate::source::model::{NodeSpan, Resolution};
use crate::source::util::find_node_span;
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TupleBindingVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    spans: Vec<NodeSpan>,
    types: HashMap<String, Option<syn::Type>>,
    tuples: HashSet<String>,
    iterators: HashMap<String, bool>,
}

impl<'context> TupleBindingVisitor<'context> {
    pub(crate) fn new(context: &'context RuleContext, ordinal: usize, module: usize) -> Self {
        Self {
            context,
            ordinal,
            module,
            spans: Vec::new(),
            types: HashMap::new(),
            tuples: HashSet::new(),
            iterators: HashMap::new(),
        }
    }
    pub(crate) fn collect_item(mut self, item: &syn::Item) -> Vec<NodeSpan> {
        self.visit_item(item);
        self.spans
    }
    pub(crate) fn collect_impl(mut self, implementation: &syn::ItemImpl) -> Vec<NodeSpan> {
        self.visit_item_impl(implementation);
        self.spans
    }

    fn tuple_return(&self, ordinal: usize, module: usize, output: &syn::ReturnType) -> bool {
        let syn::ReturnType::Type(_, typed) = output else {
            return false;
        };
        self.context.resolve_type_syntax(ordinal, module, typed).iter().any(|resolved| matches!(&resolved.typed, syn::Type::Tuple(tuple) if tuple.elems.len() >= 2))
    }

    fn call_tuple(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        if callee
            .path
            .get_ident()
            .is_some_and(|name| self.types.contains_key(&name.to_string()))
        {
            return false;
        }
        let Some(name) = callee.path.segments.last() else {
            return false;
        };
        let targets = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &callee.path);
        if !targets.is_empty() {
            return targets.iter().any(|target| target.path.item.is_some_and(|identity| {
                let item = &self.context.indexes[target.index].items[identity];
                matches!(&item.syntax, syn::Item::Fn(function) if self.tuple_return(target.index, item.module, &function.sig.output))
            }));
        }
        if ![
            "split_at",
            "split_at_mut",
            "partition",
            "unzip",
            "channel",
            "unbounded_channel",
        ]
        .iter()
        .any(|required| name.ident == *required)
        {
            return false;
        }
        let Resolution::External(path) =
            self.context.indexes[self.ordinal].resolve_path(self.module, &callee.path)
        else {
            return false;
        };
        let path = self.context.resolve_external_identity(self.ordinal, &path);
        (["tokio", "futures", "futures_channel"].contains(&path.package.as_str())
            && path.segments.iter().any(|segment| {
                ["mpsc", "oneshot", "watch", "broadcast"].contains(&segment.as_str())
            }))
            || (path.package == "std" && path.segments.iter().any(|segment| segment == "mpsc"))
    }

    fn standard_type(&self, ordinal: usize, module: usize, typed: &syn::Type) -> bool {
        match typed {
            syn::Type::Reference(reference) => self.standard_type(ordinal, module, &reference.elem),
            syn::Type::Slice(_) | syn::Type::Array(_) => true,
            syn::Type::Path(path) => {
                if path.path.is_ident("str")
                    && matches!(
                        self.context.indexes[ordinal].resolve_path(module, &path.path),
                        Resolution::Local
                    )
                {
                    return true;
                }
                self.context
                    .canonical_external_type_path(ordinal, module, &path.path)
                    .is_some_and(|identity| {
                        ["std", "alloc"].contains(&identity.package.as_str())
                            && (identity
                                .segments
                                .iter()
                                .skip(1)
                                .map(String::as_str)
                                .eq(["vec", "Vec"])
                                || identity
                                    .segments
                                    .iter()
                                    .skip(1)
                                    .map(String::as_str)
                                    .eq(["string", "String"]))
                    })
            }
            _ => false,
        }
    }

    fn standard_receiver(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(_),
                ..
            }) => true,
            syn::Expr::Paren(paren) => self.standard_receiver(&paren.expr),
            syn::Expr::Reference(reference) => self.standard_receiver(&reference.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.types.get(&name.to_string()))
                .and_then(Option::as_ref)
                .is_some_and(|typed| {
                    self.context
                        .resolve_type_syntax(self.ordinal, self.module, typed)
                        .iter()
                        .any(|resolved| {
                            self.standard_type(resolved.index, resolved.module, &resolved.typed)
                        })
                }),
            _ => false,
        }
    }

    fn parameters(&mut self, signature: &syn::Signature) {
        self.types.clear();
        self.tuples.clear();
        self.iterators.clear();
        for input in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = input
                && let syn::Pat::Ident(binding) = parameter.pat.as_ref()
            {
                self.types
                    .insert(binding.ident.to_string(), Some((*parameter.ty).clone()));
                if self.tuple_return(
                    self.ordinal,
                    self.module,
                    &syn::ReturnType::Type(Default::default(), parameter.ty.clone()),
                ) {
                    self.tuples.insert(binding.ident.to_string());
                }
            }
        }
    }

    fn collection_override(&self, collection: &str, method: &str) -> bool {
        self.context.indexes[self.ordinal].implementations.iter().any(|implementation| {
            if implementation.syntax.trait_.is_none() || !implementation.syntax.items.iter().any(|item| matches!(item, syn::ImplItem::Fn(function) if function.sig.ident == method)) {
                return false;
            }
            self.context.resolve_type_syntax(self.ordinal, implementation.module, &implementation.syntax.self_ty).iter().any(|resolved| {
                let mut typed = &resolved.typed;
                while let syn::Type::Reference(reference) = typed { typed = &reference.elem; }
                let syn::Type::Path(path) = typed else { return false; };
                let generic_receiver = path.path.get_ident().is_some_and(|name| implementation.syntax.generics.params.iter().any(|parameter| matches!(parameter, syn::GenericParam::Type(parameter) if parameter.ident == *name)));
                generic_receiver || self.context.canonical_external_type_path(resolved.index, resolved.module, &path.path).is_some_and(|identity| {
                    ["std", "alloc"].contains(&identity.package.as_str()) && match collection {
                        "Vec" => identity.segments.iter().skip(1).map(String::as_str).eq(["vec", "Vec"]),
                        "String" => identity.segments.iter().skip(1).map(String::as_str).eq(["string", "String"]),
                        _ => false,
                    }
                })
            })
        })
    }

    fn standard_iterator_type(
        &self,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        method: &str,
    ) -> bool {
        match typed {
            syn::Type::Reference(reference) => {
                self.standard_iterator_type(ordinal, module, &reference.elem, method)
            }
            syn::Type::Slice(_) | syn::Type::Array(_) => {
                ["iter", "iter_mut", "into_iter"].contains(&method)
            }
            syn::Type::Path(path)
                if path.path.is_ident("str")
                    && matches!(
                        self.context.indexes[ordinal].resolve_path(module, &path.path),
                        Resolution::Local
                    ) =>
            {
                ["chars", "lines", "bytes", "char_indices"].contains(&method)
            }
            syn::Type::Path(path) => self
                .context
                .canonical_external_type_path(ordinal, module, &path.path)
                .is_some_and(|identity| {
                    ["std", "alloc"].contains(&identity.package.as_str())
                        && match identity
                            .segments
                            .iter()
                            .skip(1)
                            .map(String::as_str)
                            .collect::<Vec<_>>()
                            .as_slice()
                        {
                            ["vec", "Vec"] => {
                                ["iter", "iter_mut", "into_iter"].contains(&method)
                                    && !self.collection_override("Vec", method)
                            }
                            ["string", "String"] => {
                                ["chars", "lines", "bytes", "char_indices"].contains(&method)
                                    && !self.collection_override("String", method)
                            }
                            _ => false,
                        }
                }),
            _ => false,
        }
    }

    fn standard_iterator_seed(&self, receiver: &syn::Expr, method: &str) -> bool {
        match receiver {
            syn::Expr::Paren(paren) => self.standard_iterator_seed(&paren.expr, method),
            syn::Expr::Reference(reference) => self.standard_iterator_seed(&reference.expr, method),
            syn::Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Str(_)) => {
                ["chars", "lines", "bytes", "char_indices"].contains(&method)
            }
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.types.get(&name.to_string()))
                .and_then(Option::as_ref)
                .is_some_and(|typed| {
                    self.context
                        .resolve_type_syntax(self.ordinal, self.module, typed)
                        .iter()
                        .any(|resolved| {
                            self.standard_iterator_type(
                                resolved.index,
                                resolved.module,
                                &resolved.typed,
                                method,
                            )
                        })
                }),
            _ => false,
        }
    }

    fn is_map_type(&self, ordinal: usize, module: usize, typed: &syn::Type) -> bool {
        match typed {
            syn::Type::Reference(reference) => self.is_map_type(ordinal, module, &reference.elem),
            syn::Type::Path(path) => {
                self.context
                    .canonical_external_type_path(ordinal, module, &path.path)
                    .is_some_and(|identity| {
                        ["std", "alloc"].contains(&identity.package.as_str())
                            && identity
                                .segments
                                .get(1)
                                .is_some_and(|name| name == "collections")
                            && identity.segments.last().is_some_and(|name| {
                                ["HashMap", "BTreeMap"].contains(&name.as_str())
                            })
                    })
            }
            _ => false,
        }
    }

    fn is_map_receiver(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_map_receiver(&paren.expr),
            syn::Expr::Reference(reference) => self.is_map_receiver(&reference.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.types.get(&name.to_string()))
                .and_then(Option::as_ref)
                .is_some_and(|typed| {
                    self.context
                        .resolve_type_syntax(self.ordinal, self.module, typed)
                        .iter()
                        .any(|resolved| {
                            self.is_map_type(resolved.index, resolved.module, &resolved.typed)
                        })
                }),
            _ => false,
        }
    }

    fn is_map_iteration(&self, call: &syn::ExprMethodCall) -> bool {
        call.args.is_empty()
            && ["iter", "iter_mut", "into_iter"]
                .iter()
                .any(|name| call.method == *name)
            && self.is_map_receiver(&call.receiver)
    }

    fn standard_iterator(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.standard_iterator(&paren.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.iterators.contains_key(&name.to_string())),
            syn::Expr::MethodCall(call) if call.args.is_empty() => {
                self.standard_iterator_seed(&call.receiver, &call.method.to_string())
                    || self.is_map_iteration(call)
                    || (["enumerate", "rev", "copied", "cloned"]
                        .iter()
                        .any(|name| call.method == *name)
                        && self.standard_iterator(&call.receiver))
            }
            syn::Expr::MethodCall(call) if call.method == "zip" && call.args.len() == 1 => {
                self.standard_iterator(&call.receiver)
            }
            _ => false,
        }
    }

    fn optional_tuple(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.optional_tuple(&paren.expr),
            syn::Expr::Group(group) => self.optional_tuple(&group.expr),
            syn::Expr::MethodCall(call) => {
                (["split_once", "rsplit_once"]
                    .iter()
                    .any(|name| call.method == *name)
                    && self.standard_receiver(&call.receiver))
                    || (["ok_or", "ok_or_else"]
                        .iter()
                        .any(|name| call.method == *name)
                        && self.optional_tuple(&call.receiver))
            }
            _ => false,
        }
    }

    fn unwrapped_tuple(&self, call: &syn::ExprMethodCall) -> bool {
        let unwraps = [
            "unwrap",
            "expect",
            "unwrap_or",
            "unwrap_or_default",
            "unwrap_or_else",
        ]
        .iter()
        .any(|name| call.method == *name);
        unwraps && self.optional_tuple(&call.receiver)
    }

    fn tuple_block(&self, block: &syn::Block) -> bool {
        matches!(
            block.stmts.last(),
            Some(syn::Stmt::Expr(expression, None)) if self.tuple_expression(expression)
        )
    }

    fn tuple_expression(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Tuple(tuple) => tuple.elems.len() >= 2,
            syn::Expr::Paren(paren) => self.tuple_expression(&paren.expr),
            syn::Expr::Group(group) => self.tuple_expression(&group.expr),
            syn::Expr::Try(attempt) => self.optional_tuple(&attempt.expr),
            syn::Expr::Block(block) => self.tuple_block(&block.block),
            syn::Expr::If(branch) => {
                self.tuple_block(&branch.then_branch)
                    || branch
                        .else_branch
                        .as_ref()
                        .is_some_and(|(_, other)| self.tuple_expression(other))
            }
            syn::Expr::Match(matched) => matched
                .arms
                .iter()
                .any(|arm| self.tuple_expression(&arm.body)),
            syn::Expr::Call(call) => self.call_tuple(call),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .is_some_and(|name| self.tuples.contains(&name.to_string())),
            syn::Expr::MethodCall(call) => {
                (["split_at", "split_at_mut"]
                    .iter()
                    .any(|name| call.method == *name)
                    && self.standard_receiver(&call.receiver))
                    || (["partition", "unzip", "size_hint"]
                        .iter()
                        .any(|name| call.method == *name)
                        && self.standard_iterator(&call.receiver))
                    || self.unwrapped_tuple(call)
            }
            _ => false,
        }
    }

    fn tuple_iterator(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.tuple_iterator(&paren.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.iterators.get(&name.to_string()))
                .copied()
                .unwrap_or(false),
            syn::Expr::MethodCall(call) if call.method == "enumerate" && call.args.is_empty() => {
                self.standard_iterator(&call.receiver)
            }
            syn::Expr::MethodCall(call) if call.method == "rev" && call.args.is_empty() => {
                self.tuple_iterator(&call.receiver)
            }
            syn::Expr::MethodCall(call)
                if call.method == "char_indices" && call.args.is_empty() =>
            {
                self.standard_iterator_seed(&call.receiver, "char_indices")
            }
            syn::Expr::MethodCall(call) if self.is_map_iteration(call) => true,
            syn::Expr::MethodCall(call) if call.method == "zip" && call.args.len() == 1 => {
                self.standard_iterator(&call.receiver)
            }
            _ => false,
        }
    }

    fn named_binding(pattern: &syn::Pat) -> bool {
        match pattern {
            syn::Pat::Ident(_) => true,
            syn::Pat::Type(typed) => Self::named_binding(&typed.pat),
            _ => false,
        }
    }

    fn named_item_closure<'call>(
        &self,
        call: &'call syn::ExprMethodCall,
    ) -> Option<&'call syn::Pat> {
        let adapts_items = [
            "map",
            "filter",
            "for_each",
            "filter_map",
            "flat_map",
            "inspect",
            "take_while",
            "skip_while",
            "map_while",
            "find",
            "find_map",
            "position",
            "any",
            "all",
            "max_by_key",
            "min_by_key",
        ]
        .iter()
        .any(|name| call.method == *name);
        if !adapts_items || call.args.len() != 1 || !self.tuple_iterator(&call.receiver) {
            return None;
        }
        let Some(syn::Expr::Closure(closure)) = call.args.first() else {
            return None;
        };
        closure
            .inputs
            .first()
            .filter(|pattern| closure.inputs.len() == 1 && Self::named_binding(pattern))
    }
}

impl<'ast> Visit<'ast> for TupleBindingVisitor<'_> {
    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        if matches!(expression.pat.as_ref(), syn::Pat::Ident(_))
            && self.tuple_iterator(&expression.expr)
        {
            self.spans.push(find_node_span(expression.pat.span()));
        }
        self.visit_expr(&expression.expr);
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        self.visit_pat(&expression.pat);
        self.visit_block(&expression.body);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if let Some(pattern) = self.named_item_closure(call) {
            self.spans.push(find_node_span(pattern.span()));
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.visit_pat(&expression.pat);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
        if let Some((_, branch)) = &expression.else_branch {
            self.visit_expr(branch);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.body);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
    }

    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.types.insert(pattern.ident.to_string(), None);
        self.tuples.remove(&pattern.ident.to_string());
        self.iterators.remove(&pattern.ident.to_string());
        syn::visit::visit_pat_ident(self, pattern);
    }

    fn visit_pat_type(&mut self, pattern: &'ast syn::PatType) {
        syn::visit::visit_pat_type(self, pattern);
        if let syn::Pat::Ident(binding) = pattern.pat.as_ref() {
            self.types
                .insert(binding.ident.to_string(), Some((*pattern.ty).clone()));
        }
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        syn::visit::visit_expr_closure(self, closure);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
    }

    fn visit_arm(&mut self, arm: &'ast syn::Arm) {
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        syn::visit::visit_arm(self, arm);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        self.parameters(&function.sig);
        self.visit_block(&function.block);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.parameters(&function.sig);
        self.visit_block(&function.block);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let types = self.types.clone();
        let tuples = self.tuples.clone();
        let iterators = self.iterators.clone();
        syn::visit::visit_block(self, block);
        self.types = types;
        self.tuples = tuples;
        self.iterators = iterators;
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        let iterator = local.init.as_ref().and_then(|initializer| {
            self.standard_iterator(&initializer.expr)
                .then(|| self.tuple_iterator(&initializer.expr))
        });
        let named = match &local.pat {
            syn::Pat::Ident(_) => true,
            syn::Pat::Type(typed) => matches!(typed.pat.as_ref(), syn::Pat::Ident(_)),
            _ => false,
        };
        let tuple = local
            .init
            .as_ref()
            .is_some_and(|initializer| self.tuple_expression(&initializer.expr));
        if named && tuple {
            self.spans.push(find_node_span(local.span()));
        }
        let inherited = local.init.as_ref().and_then(|initializer| {
            let syn::Expr::Path(path) = initializer.expr.as_ref() else {
                return None;
            };
            path.path
                .get_ident()
                .and_then(|name| self.types.get(&name.to_string()))
                .cloned()
                .flatten()
        });
        if let Some(initializer) = &local.init {
            self.visit_expr(&initializer.expr);
            if let Some((_, diverging)) = &initializer.diverge {
                self.visit_expr(diverging);
            }
        }
        self.visit_pat(&local.pat);
        let (binding, annotation) = match &local.pat {
            syn::Pat::Ident(binding) => (binding, None),
            syn::Pat::Type(typed) => {
                let syn::Pat::Ident(binding) = typed.pat.as_ref() else {
                    return;
                };
                (binding, Some((*typed.ty).clone()))
            }
            _ => return,
        };
        let name = binding.ident.to_string();
        self.tuples.remove(&name);
        self.types.insert(name.clone(), annotation.or(inherited));
        if let Some(tuple) = iterator {
            self.iterators.insert(name.clone(), tuple);
        }
        if tuple {
            self.tuples.insert(name);
        }
    }
}
