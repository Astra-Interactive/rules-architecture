use crate::check::model::RuleContext;
use crate::source::model::Resolution;
use std::collections::BTreeMap;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct PanicDocVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    cfg: crate::source::model::CfgPredicate,
    parameters: BTreeMap<String, syn::Type>,
    nested: BTreeMap<String, syn::ItemFn>,
    self_type: Option<Box<syn::Type>>,
    found: bool,
}

impl PanicDocVisitor<'_> {
    fn signature_parameters(signature: &syn::Signature) -> BTreeMap<String, syn::Type> {
        signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(parameter) => {
                    if let syn::Pat::Ident(binding) = parameter.pat.as_ref() {
                        Some((binding.ident.to_string(), (*parameter.ty).clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect()
    }

    pub(crate) fn contains_panic(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        signature: &syn::Signature,
        cfg: &crate::source::model::CfgPredicate,
        block: &syn::Block,
    ) -> bool {
        let parameters = Self::signature_parameters(signature);
        let self_type = context.indexes[ordinal]
            .implementations
            .iter()
            .find(|implementation| {
                implementation.module == module
                    && implementation.syntax.items.iter().any(|item| {
                        matches!(item, syn::ImplItem::Fn(method) if crate::source::util::find_node_span(method.sig.span()) == crate::source::util::find_node_span(signature.span()))
                    })
            })
            .map(|implementation| implementation.syntax.self_ty.clone());
        let mut visitor = PanicDocVisitor {
            context,
            ordinal,
            module,
            cfg: cfg.clone(),
            parameters,
            nested: BTreeMap::new(),
            self_type,
            found: false,
        };
        visitor.visit_block(block);
        visitor.found
    }

    fn standard_receiver(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = expression else {
            return false;
        };
        let Some(name) = path.path.get_ident() else {
            return false;
        };
        let Some(typed) = self.parameters.get(&name.to_string()) else {
            return false;
        };
        let workspace = self
            .context
            .resolve_outer_workspace_type(self.ordinal, self.module, typed)
            .iter()
            .any(|target| self.cfg.and(&target.path.cfg).is_satisfiable());
        !workspace
            && self
                .context
                .resolve_outer_external_type_paths_under(
                    self.ordinal,
                    self.module,
                    typed,
                    &self.cfg,
                )
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["Option", "Result"].contains(&name.as_str()))
                })
    }

    fn literal_index(expression: &syn::Expr) -> Option<u128> {
        match expression {
            syn::Expr::Lit(literal) => match &literal.lit {
                syn::Lit::Int(integer) => integer.base10_parse().ok(),
                _ => None,
            },
            syn::Expr::Paren(paren) => Self::literal_index(&paren.expr),
            syn::Expr::Group(group) => Self::literal_index(&group.expr),
            _ => None,
        }
    }

    fn safe_array_index(array: &syn::TypeArray, index: &syn::Expr) -> bool {
        matches!((Self::literal_index(&array.len), Self::literal_index(index)), (Some(length), Some(index)) if index < length)
    }

    fn safe_range(index: &syn::Expr, length: Option<u128>) -> bool {
        let syn::Expr::Range(range) = index else {
            return false;
        };
        let start = match range.start.as_deref() {
            Some(start) => Self::literal_index(start),
            None => Some(0),
        };
        let end = match range.end.as_deref() {
            Some(end) => Self::literal_index(end).and_then(|end| match range.limits {
                syn::RangeLimits::Closed(_) => end.checked_add(1),
                syn::RangeLimits::HalfOpen(_) => Some(end),
            }),
            None if start == Some(0) => return true,
            None => length,
        };
        matches!((start, end), (Some(start), Some(end)) if start <= end && end <= length.unwrap_or(0))
    }

    fn safe_index(typed: &syn::Type, index: &syn::Expr) -> bool {
        if let syn::Expr::Paren(paren) = index {
            return Self::safe_index(typed, &paren.expr);
        }
        if let syn::Expr::Group(group) = index {
            return Self::safe_index(typed, &group.expr);
        }
        match typed {
            syn::Type::Reference(reference) => Self::safe_index(&reference.elem, index),
            syn::Type::Slice(_) => Self::safe_range(index, None),
            syn::Type::Array(array) => {
                Self::safe_array_index(array, index)
                    || Self::safe_range(index, Self::literal_index(&array.len))
            }
            _ => true,
        }
    }

    fn is_indexable_std_type(&self, index: usize, module: usize, path: &syn::Path) -> bool {
        if path.is_ident("str") {
            return true;
        }
        self.context
            .canonical_external_type_path(index, module, path)
            .is_some_and(|identity| {
                let tail: Vec<&str> = identity
                    .segments
                    .iter()
                    .skip(1)
                    .map(String::as_str)
                    .collect();
                let is_keyed_collection = tail.first() == Some(&"collections")
                    && tail
                        .last()
                        .is_some_and(|name| ["HashMap", "BTreeMap", "VecDeque"].contains(name));
                ["std", "alloc"].contains(&identity.package.as_str())
                    && (tail == ["vec", "Vec"]
                        || tail == ["string", "String"]
                        || is_keyed_collection)
            })
    }

    fn is_primitive_integer(typed: &syn::Type) -> bool {
        const INTEGERS: [&str; 12] = [
            "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        ];
        let syn::Type::Path(path) = typed else {
            return false;
        };
        path.qself.is_none()
            && path
                .path
                .get_ident()
                .is_some_and(|name| INTEGERS.contains(&name.to_string().as_str()))
    }

    fn is_integer_parameter(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Paren(paren) => self.is_integer_parameter(&paren.expr),
            syn::Expr::Group(group) => self.is_integer_parameter(&group.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.parameters.get(&name.to_string()))
                .is_some_and(Self::is_primitive_integer),
            _ => false,
        }
    }

    fn visit_nested_call(&mut self, name: &str) {
        let Some(function) = self.nested.remove(name) else {
            return;
        };
        let parameters = std::mem::replace(
            &mut self.parameters,
            Self::signature_parameters(&function.sig),
        );
        self.visit_block(&function.block);
        self.parameters = parameters;
        self.nested.insert(name.to_owned(), function);
    }

    fn visit_argument(&mut self, argument: &syn::Expr) {
        let syn::Expr::Closure(closure) = argument else {
            self.visit_expr(argument);
            return;
        };
        let parameters = self.parameters.clone();
        for input in &closure.inputs {
            self.visit_pat(input);
        }
        self.visit_expr(&closure.body);
        self.parameters = parameters;
    }

    fn has_index_panic(
        &self,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        index: &syn::Expr,
    ) -> bool {
        if let syn::Expr::Paren(paren) = index {
            return self.has_index_panic(ordinal, module, typed, &paren.expr);
        }
        if let syn::Expr::Group(group) = index {
            return self.has_index_panic(ordinal, module, typed, &group.expr);
        }
        self.context
            .resolve_type_syntax(ordinal, module, typed)
            .iter()
            .any(|resolved| {
                if resolved.has_unsubstituted_parameters {
                    return false;
                }
                let compatible = self.cfg.and(&resolved.cfg).is_satisfiable();
                if !compatible {
                    return false;
                }
                let mut typed = &resolved.typed;
                while let syn::Type::Reference(reference) = typed {
                    typed = &reference.elem;
                }
                if let syn::Type::Path(path) = typed {
                    return self.is_indexable_std_type(resolved.index, resolved.module, &path.path)
                        && !Self::safe_range(index, None);
                }
                !Self::safe_index(typed, index)
            })
    }

    fn has_field_index_panic(&self, field: &syn::ExprField, index: &syn::Expr) -> bool {
        if !matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")) {
            return false;
        }
        let Some(typed) = &self.self_type else {
            return false;
        };
        self.context
            .resolve_outer_workspace_type(self.ordinal, self.module, typed)
            .iter()
            .any(|owner| {
                if !self.cfg.and(&owner.path.cfg).is_satisfiable() {
                    return false;
                }
                let Some(identity) = owner.path.item else {
                    return false;
                };
                let item = &self.context.indexes[owner.index].items[identity];
                let syn::Item::Struct(structure) = &item.syntax else {
                    return false;
                };
                let member = match &field.member {
                    syn::Member::Named(name) => structure
                        .fields
                        .iter()
                        .find(|member| member.ident.as_ref() == Some(name)),
                    syn::Member::Unnamed(position) => {
                        structure.fields.iter().nth(position.index as usize)
                    }
                };
                member.is_some_and(|member| {
                    self.cfg
                        .and(&crate::source::model::CfgPredicate::from_attributes(
                            &member.attrs,
                        ))
                        .is_satisfiable()
                        && self.has_index_panic(owner.index, item.module, &member.ty, index)
                })
            })
    }
}

impl<'ast> Visit<'ast> for PanicDocVisitor<'_> {
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let parameters = self.parameters.clone();
        let nested = self.nested.clone();
        for statement in &block.stmts {
            if let syn::Stmt::Item(syn::Item::Fn(function)) = statement {
                self.nested
                    .insert(function.sig.ident.to_string(), function.clone());
            }
        }
        syn::visit::visit_block(self, block);
        self.parameters = parameters;
        self.nested = nested;
    }

    fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
        self.found |= matches!(
            expression.op,
            syn::BinOp::Div(_)
                | syn::BinOp::Rem(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
        ) && self.is_integer_parameter(&expression.right);
        syn::visit::visit_expr_binary(self, expression);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(initializer) = &local.init {
            self.visit_expr(&initializer.expr);
        }
        self.visit_pat(&local.pat);
    }

    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.parameters.remove(&pattern.ident.to_string());
        syn::visit::visit_pat_ident(self, pattern);
    }

    fn visit_expr_index(&mut self, expression: &'ast syn::ExprIndex) {
        if let syn::Expr::Path(path) = expression.expr.as_ref()
            && let Some(name) = path.path.get_ident()
            && let Some(typed) = self.parameters.get(&name.to_string())
        {
            self.found |= self.has_index_panic(self.ordinal, self.module, typed, &expression.index);
        }
        if let syn::Expr::Field(field) = expression.expr.as_ref() {
            self.found |= self.has_field_index_panic(field, &expression.index);
        }
        syn::visit::visit_expr_index(self, expression);
    }

    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        let names = [
            "panic",
            "assert",
            "assert_eq",
            "assert_ne",
            "debug_assert",
            "debug_assert_eq",
            "debug_assert_ne",
            "unreachable",
            "todo",
            "unimplemented",
        ];
        self.found |=
            match self.context.indexes[self.ordinal].resolve_path(self.module, &expression.path) {
                Resolution::External(path) => {
                    let identity = self.context.resolve_external_identity(self.ordinal, &path);
                    ["std", "core"].contains(&identity.package.as_str())
                        && identity.segments.len() == 2
                        && identity
                            .segments
                            .last()
                            .is_some_and(|name| names.contains(&name.as_str()))
                }
                Resolution::Local | Resolution::Unresolved => expression
                    .path
                    .get_ident()
                    .is_some_and(|name| names.iter().any(|builtin| name == builtin)),
                _ => false,
            };
        for argument in &crate::source::util::parse_macro_arguments(expression) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            self.found |= self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| {
                    ["std", "core"].contains(&path.package.as_str())
                        && path
                            .segments
                            .iter()
                            .any(|name| ["Option", "Result"].contains(&name.as_str()))
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["unwrap", "expect"].contains(&name.as_str()))
                });
        }
        let callee = match call.func.as_ref() {
            syn::Expr::Path(path) => path.path.get_ident(),
            _ => None,
        };
        if let Some(name) = callee {
            self.visit_nested_call(&name.to_string());
        }
        self.visit_expr(&call.func);
        for argument in &call.args {
            self.visit_argument(argument);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let is_panicking_method = ["unwrap", "expect", "unwrap_err", "expect_err"]
            .iter()
            .any(|name| call.method == *name);
        self.found |= is_panicking_method && self.standard_receiver(&call.receiver);
        self.visit_expr(&call.receiver);
        for argument in &call.args {
            self.visit_argument(argument);
        }
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
    fn visit_expr_async(&mut self, _: &'ast syn::ExprAsync) {}
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
}
