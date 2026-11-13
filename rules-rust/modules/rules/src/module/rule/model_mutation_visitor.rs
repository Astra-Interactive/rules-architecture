use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::is_model_item;
use crate::source::model::WorkspaceResolvedPath;
use crate::source::util::find_node_span;
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ModelMutationVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    file: &'context str,
    bindings: BTreeMap<String, Vec<WorkspaceResolvedPath>>,
    arguments: BTreeMap<String, Vec<Vec<WorkspaceResolvedPath>>>,
    model_places: BTreeSet<String>,
    findings: Vec<RuleFinding>,
}

impl<'context> ModelMutationVisitor<'context> {
    fn resolve_binding(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<WorkspaceResolvedPath> {
        let outer = self
            .context
            .resolve_outer_workspace_type(index, module, typed);
        if outer.is_empty() {
            self.context.resolve_workspace_type(index, module, typed)
        } else {
            outer
        }
    }

    fn list_arguments(
        &self,
        index: usize,
        module: usize,
        typed: &syn::Type,
    ) -> Vec<Vec<WorkspaceResolvedPath>> {
        match typed {
            syn::Type::Reference(reference) => self.list_arguments(index, module, &reference.elem),
            syn::Type::Paren(paren) => self.list_arguments(index, module, &paren.elem),
            syn::Type::Group(group) => self.list_arguments(index, module, &group.elem),
            syn::Type::Slice(slice) => vec![self.resolve_binding(index, module, &slice.elem)],
            syn::Type::Array(array) => vec![self.resolve_binding(index, module, &array.elem)],
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map(|segment| match &segment.arguments {
                    syn::PathArguments::AngleBracketed(arguments) => arguments
                        .args
                        .iter()
                        .filter_map(|argument| match argument {
                            syn::GenericArgument::Type(inner) => {
                                Some(self.resolve_binding(index, module, inner))
                            }
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                })
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    fn bind_pattern(
        &mut self,
        pattern: &syn::Pat,
        targets: Vec<WorkspaceResolvedPath>,
        arguments: Vec<Vec<WorkspaceResolvedPath>>,
    ) {
        match pattern {
            syn::Pat::Ident(binding) => {
                let name = binding.ident.to_string();
                self.model_places.remove(&name);
                self.bindings.insert(name.clone(), targets);
                self.arguments.insert(name, arguments);
                if let Some((_, nested)) = &binding.subpat {
                    self.bind_pattern(nested, Vec::new(), Vec::new());
                }
            }
            syn::Pat::Type(typed) => self.bind_pattern(&typed.pat, targets, arguments),
            syn::Pat::Tuple(tuple) => self.bind_elements(tuple.elems.iter(), &arguments),
            syn::Pat::TupleStruct(tuple) => self.bind_elements(tuple.elems.iter(), &arguments),
            syn::Pat::Struct(structure) => self.bind_elements(
                structure.fields.iter().map(|field| field.pat.as_ref()),
                &arguments,
            ),
            syn::Pat::Slice(slice) => self.bind_elements(slice.elems.iter(), &arguments),
            syn::Pat::Reference(reference) => self.bind_pattern(&reference.pat, targets, arguments),
            syn::Pat::Paren(paren) => self.bind_pattern(&paren.pat, targets, arguments),
            syn::Pat::Guard(guard) => {
                self.bind_pattern(&guard.pat, targets, arguments);
                self.visit_expr(&guard.guard);
            }
            syn::Pat::Or(alternatives) => {
                for alternative in &alternatives.cases {
                    self.bind_pattern(alternative, Vec::new(), Vec::new());
                }
            }
            _ => {}
        }
    }

    fn bind_elements<'pattern>(
        &mut self,
        elements: impl Iterator<Item = &'pattern syn::Pat>,
        arguments: &[Vec<WorkspaceResolvedPath>],
    ) {
        for element in elements {
            self.bind_pattern(element, arguments.concat(), Vec::new());
        }
    }

    pub(crate) fn inspect(
        context: &'context RuleContext,
        ordinal: usize,
        module: usize,
        file: &'context str,
        signature: &syn::Signature,
        block: &syn::Block,
        self_type: Option<&syn::Type>,
    ) -> Vec<RuleFinding> {
        let mut visitor = Self {
            context,
            ordinal,
            module,
            file,
            bindings: BTreeMap::new(),
            arguments: BTreeMap::new(),
            model_places: BTreeSet::new(),
            findings: Vec::new(),
        };
        if let Some(typed) = self_type {
            visitor.bindings.insert(
                "self".to_owned(),
                visitor.resolve_binding(ordinal, module, typed),
            );
            visitor.arguments.insert(
                "self".to_owned(),
                visitor.list_arguments(ordinal, module, typed),
            );
        }
        for parameter in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = parameter {
                visitor.bind_pattern(
                    &parameter.pat,
                    visitor.resolve_binding(ordinal, module, &parameter.ty),
                    visitor.list_arguments(ordinal, module, &parameter.ty),
                );
            }
        }
        visitor.visit_block(block);
        visitor.findings
    }

    fn find_field<'structure>(
        structure: &'structure syn::ItemStruct,
        member: &syn::Member,
    ) -> Option<&'structure syn::Field> {
        match member {
            syn::Member::Named(name) => structure
                .fields
                .iter()
                .find(|field| field.ident.as_ref() == Some(name)),
            syn::Member::Unnamed(position) => structure.fields.iter().nth(position.index as usize),
        }
    }

    fn find_parameter_position(structure: &syn::ItemStruct, typed: &syn::Type) -> Option<usize> {
        let syn::Type::Path(path) = typed else {
            return None;
        };
        let name = path.path.get_ident()?;
        structure
            .generics
            .params
            .iter()
            .filter_map(|parameter| match parameter {
                syn::GenericParam::Type(parameter) => Some(&parameter.ident),
                _ => None,
            })
            .position(|parameter| parameter == name)
    }

    fn field_targets(&self, field: &syn::ExprField) -> Vec<WorkspaceResolvedPath> {
        let mut targets = Vec::new();
        for owner in self.targets(&field.base) {
            let Some(identity) = owner.path.item else {
                continue;
            };
            let item = &self.context.indexes[owner.index].items[identity];
            let syn::Item::Struct(structure) = &item.syntax else {
                continue;
            };
            let Some(declared) = Self::find_field(structure, &field.member) else {
                continue;
            };
            match Self::find_parameter_position(structure, &declared.ty) {
                Some(position) => targets.extend(
                    self.arguments_of(&field.base)
                        .get(position)
                        .cloned()
                        .unwrap_or_default(),
                ),
                None => {
                    targets.extend(self.resolve_binding(owner.index, item.module, &declared.ty))
                }
            }
        }
        targets
    }

    fn field_arguments(&self, field: &syn::ExprField) -> Vec<Vec<WorkspaceResolvedPath>> {
        self.targets(&field.base)
            .iter()
            .find_map(|owner| {
                let item = &self.context.indexes[owner.index].items[owner.path.item?];
                let syn::Item::Struct(structure) = &item.syntax else {
                    return None;
                };
                let declared = Self::find_field(structure, &field.member)?;
                Some(self.list_arguments(owner.index, item.module, &declared.ty))
            })
            .unwrap_or_default()
    }

    fn arguments_of(&self, expression: &syn::Expr) -> Vec<Vec<WorkspaceResolvedPath>> {
        match expression {
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.arguments.get(&name.to_string()))
                .cloned()
                .unwrap_or_default(),
            syn::Expr::Reference(reference) => self.arguments_of(&reference.expr),
            syn::Expr::Unary(unary) => self.arguments_of(&unary.expr),
            syn::Expr::Paren(paren) => self.arguments_of(&paren.expr),
            syn::Expr::Field(field) => self.field_arguments(field),
            syn::Expr::MethodCall(call)
                if Self::ELEMENT_ITERATORS.contains(&call.method.to_string().as_str()) =>
            {
                self.arguments_of(&call.receiver)
            }
            _ => Vec::new(),
        }
    }

    fn is_model(&self, target: &WorkspaceResolvedPath) -> bool {
        target.path.item.is_some_and(|identity| {
            is_model_item(
                self.context,
                target.index,
                &self.context.indexes[target.index].items[identity],
            )
        })
    }

    fn call_targets(&self, call: &syn::ExprCall) -> Vec<WorkspaceResolvedPath> {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return Vec::new();
        };
        self.context
            .resolve_workspace_path(self.ordinal, self.module, &callee.path)
            .into_iter()
            .filter(|target| target.index == self.ordinal && target.path.cfg.is_satisfiable())
            .filter_map(|target| {
                let item = &self.context.indexes[target.index].items[target.path.item?];
                let syn::Item::Fn(function) = &item.syntax else {
                    return None;
                };
                let syn::ReturnType::Type(_, returned) = &function.sig.output else {
                    return None;
                };
                Some(self.resolve_binding(target.index, item.module, returned))
            })
            .flatten()
            .filter(|target| self.is_model(target))
            .collect()
    }

    fn targets(&self, expression: &syn::Expr) -> Vec<WorkspaceResolvedPath> {
        match expression {
            syn::Expr::Call(call) => self.call_targets(call),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.bindings.get(&name.to_string()))
                .cloned()
                .unwrap_or_else(|| {
                    self.context
                        .resolve_workspace_path(self.ordinal, self.module, &path.path)
                }),
            syn::Expr::Reference(reference) => self.targets(&reference.expr),
            syn::Expr::Unary(unary) => self.targets(&unary.expr),
            syn::Expr::Paren(paren) => self.targets(&paren.expr),
            syn::Expr::Field(field) => self.field_targets(field),
            syn::Expr::Struct(structure) => {
                self.context
                    .resolve_workspace_path(self.ordinal, self.module, &structure.path)
            }
            _ => Vec::new(),
        }
    }

    fn model_place(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Field(field) => self
                .targets(&field.base)
                .iter()
                .any(|target| self.is_model(target)),
            syn::Expr::Index(index) => self.model_place(&index.expr),
            syn::Expr::MethodCall(call)
                if Self::MUTABLE_ACCESSORS.contains(&call.method.to_string().as_str()) =>
            {
                self.model_place(&call.receiver)
            }
            syn::Expr::Reference(reference) => self.model_place(&reference.expr),
            syn::Expr::Paren(paren) => self.model_place(&paren.expr),
            syn::Expr::Unary(unary) => match unary.expr.as_ref() {
                syn::Expr::Path(path) => path
                    .path
                    .get_ident()
                    .is_some_and(|name| self.model_places.contains(&name.to_string())),
                expression => self.model_place(expression),
            },
            _ => false,
        }
    }

    fn collection_field(expression: &syn::Expr) -> Option<&syn::ExprField> {
        match expression {
            syn::Expr::Field(field) => Some(field),
            syn::Expr::Reference(reference) => Self::collection_field(&reference.expr),
            syn::Expr::Paren(paren) => Self::collection_field(&paren.expr),
            _ => None,
        }
    }

    fn is_collection_type(typed: &syn::Type) -> bool {
        match typed {
            syn::Type::Paren(paren) => Self::is_collection_type(&paren.elem),
            syn::Type::Group(group) => Self::is_collection_type(&group.elem),
            syn::Type::Path(path) => path.path.segments.last().is_some_and(|segment| {
                Self::COLLECTION_TYPES.contains(&segment.ident.to_string().as_str())
            }),
            _ => false,
        }
    }

    fn has_collection_field(&self, field: &syn::ExprField) -> bool {
        self.targets(&field.base).iter().any(|owner| {
            let Some(identity) = owner.path.item else {
                return false;
            };
            let item = &self.context.indexes[owner.index].items[identity];
            let syn::Item::Struct(structure) = &item.syntax else {
                return false;
            };
            Self::find_field(structure, &field.member)
                .is_some_and(|declared| Self::is_collection_type(&declared.ty))
        })
    }

    fn push_finding(&mut self, span: proc_macro2::Span) {
        self.findings.push(build_node_finding(
            "ModelShouldBeImmutable",
            self.file,
            find_node_span(span),
            "model field".to_owned(),
            "a model transition returns a new value instead of assigning a field".to_owned(),
        ));
    }

    fn is_option_type(typed: &syn::Type) -> bool {
        match typed {
            syn::Type::Paren(paren) => Self::is_option_type(&paren.elem),
            syn::Type::Group(group) => Self::is_option_type(&group.elem),
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Option"),
            _ => false,
        }
    }

    fn has_option_field(&self, field: &syn::ExprField) -> bool {
        self.targets(&field.base).iter().any(|owner| {
            let Some(identity) = owner.path.item else {
                return false;
            };
            let item = &self.context.indexes[owner.index].items[identity];
            let syn::Item::Struct(structure) = &item.syntax else {
                return false;
            };
            Self::find_field(structure, &field.member)
                .is_some_and(|declared| Self::is_option_type(&declared.ty))
        })
    }

    fn report_collection_mutation(&mut self, call: &syn::ExprMethodCall) {
        let Some(field) = Self::collection_field(&call.receiver) else {
            return;
        };
        let method = call.method.to_string();
        let is_model_field = self
            .targets(&field.base)
            .iter()
            .any(|target| self.is_model(target));
        let is_collection_write = Self::COLLECTION_MUTATORS.contains(&method.as_str())
            && self.has_collection_field(field);
        let is_option_write =
            Self::OPTION_MUTATORS.contains(&method.as_str()) && self.has_option_field(field);
        if is_model_field && (is_collection_write || is_option_write) {
            self.push_finding(call.span());
        }
    }

    fn is_memory_exchange(&self, callee: &syn::Expr) -> bool {
        let syn::Expr::Path(path) = callee else {
            return false;
        };
        let crate::source::model::Resolution::External(resolved) =
            self.context.indexes[self.ordinal].resolve_path(self.module, &path.path)
        else {
            return false;
        };
        [
            "std::mem::replace",
            "std::mem::swap",
            "std::mem::take",
            "core::mem::replace",
            "core::mem::swap",
            "core::mem::take",
        ]
        .contains(&resolved.as_str())
    }

    fn report_memory_exchange(&mut self, call: &syn::ExprCall) {
        if !self.is_memory_exchange(&call.func) {
            return;
        }
        let writes_model = call.args.iter().any(|argument| {
            matches!(argument, syn::Expr::Reference(reference)
                if reference.mutability.is_some() && self.model_place(&reference.expr))
        });
        if writes_model {
            self.push_finding(call.span());
        }
    }

    fn report_write(&mut self, target: &syn::Expr, span: proc_macro2::Span) {
        if self.model_place(target) {
            self.push_finding(span);
        }
    }

    const COLLECTION_TYPES: &'static [&'static str] = &[
        "Vec",
        "VecDeque",
        "HashMap",
        "HashSet",
        "BTreeMap",
        "BTreeSet",
        "BinaryHeap",
    ];

    const COLLECTION_MUTATORS: &'static [&'static str] = &[
        "push",
        "push_back",
        "push_front",
        "pop",
        "pop_back",
        "pop_front",
        "insert",
        "remove",
        "clear",
        "extend",
        "append",
        "retain",
        "truncate",
        "drain",
        "dedup",
        "reverse",
        "sort",
        "sort_by",
        "sort_by_key",
        "sort_unstable",
        "swap_remove",
        "resize",
        "split_off",
    ];

    const OPTION_MUTATORS: &'static [&'static str] = &[
        "take",
        "replace",
        "insert",
        "get_or_insert",
        "get_or_insert_with",
    ];

    const MUTABLE_ACCESSORS: &'static [&'static str] = &[
        "get_mut",
        "iter_mut",
        "first_mut",
        "last_mut",
        "values_mut",
        "as_mut",
        "as_deref_mut",
        "unwrap",
        "expect",
    ];

    const ELEMENT_ITERATORS: &'static [&'static str] = &[
        "iter",
        "iter_mut",
        "into_iter",
        "drain",
        "values",
        "values_mut",
    ];
}

impl<'ast> Visit<'ast> for ModelMutationVisitor<'_> {
    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let bindings = self.bindings.clone();
        let arguments = self.arguments.clone();
        let places = self.model_places.clone();
        self.bind_pattern(
            &expression.pat,
            self.arguments_of(&expression.expr).concat(),
            Vec::new(),
        );
        if let syn::Pat::Ident(binding) = expression.pat.as_ref()
            && self.model_place(&expression.expr)
        {
            self.model_places.insert(binding.ident.to_string());
        }
        self.visit_block(&expression.body);
        self.bindings = bindings;
        self.arguments = arguments;
        self.model_places = places;
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        for arm in &expression.arms {
            let bindings = self.bindings.clone();
            let arguments = self.arguments.clone();
            let places = self.model_places.clone();
            self.bind_pattern(
                &arm.pat,
                self.targets(&expression.expr),
                self.arguments_of(&expression.expr),
            );
            self.visit_expr(&arm.body);
            self.bindings = bindings;
            self.arguments = arguments;
            self.model_places = places;
        }
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.bind_pattern(
            &expression.pat,
            self.targets(&expression.expr),
            self.arguments_of(&expression.expr),
        );
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let bindings = self.bindings.clone();
        let arguments = self.arguments.clone();
        let places = self.model_places.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings = bindings;
        self.arguments = arguments;
        self.model_places = places;
        if let Some((_, expression)) = &expression.else_branch {
            self.visit_expr(expression);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let bindings = self.bindings.clone();
        let arguments = self.arguments.clone();
        let places = self.model_places.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings = bindings;
        self.arguments = arguments;
        self.model_places = places;
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let bindings = self.bindings.clone();
        let arguments = self.arguments.clone();
        let places = self.model_places.clone();
        for parameter in &closure.inputs {
            match parameter {
                syn::Pat::Type(typed) => self.bind_pattern(
                    parameter,
                    self.resolve_binding(self.ordinal, self.module, &typed.ty),
                    self.list_arguments(self.ordinal, self.module, &typed.ty),
                ),
                _ => self.bind_pattern(parameter, Vec::new(), Vec::new()),
            }
        }
        self.visit_expr(&closure.body);
        self.bindings = bindings;
        self.arguments = arguments;
        self.model_places = places;
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.report_collection_mutation(call);
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        self.report_memory_exchange(call);
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_assign(&mut self, expression: &'ast syn::ExprAssign) {
        self.report_write(&expression.left, expression.span());
        syn::visit::visit_expr_assign(self, expression);
    }

    fn visit_expr_binary(&mut self, expression: &'ast syn::ExprBinary) {
        if matches!(
            expression.op,
            syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
        ) {
            self.report_write(&expression.left, expression.span());
        }
        syn::visit::visit_expr_binary(self, expression);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        syn::visit::visit_local(self, local);
        let targets = match &local.pat {
            syn::Pat::Type(typed) => self.resolve_binding(self.ordinal, self.module, &typed.ty),
            _ => local
                .init
                .as_ref()
                .map(|initializer| self.targets(&initializer.expr))
                .unwrap_or_default(),
        };
        let arguments = match &local.pat {
            syn::Pat::Type(typed) => self.list_arguments(self.ordinal, self.module, &typed.ty),
            _ => local
                .init
                .as_ref()
                .map(|initializer| self.arguments_of(&initializer.expr))
                .unwrap_or_default(),
        };
        let pattern = match &local.pat {
            syn::Pat::Type(typed) => typed.pat.as_ref(),
            pattern => pattern,
        };
        let place = local
            .init
            .as_ref()
            .is_some_and(|initializer| self.model_place(&initializer.expr));
        self.bind_pattern(&local.pat, targets, arguments);
        if let syn::Pat::Ident(binding) = pattern
            && place
        {
            self.model_places.insert(binding.ident.to_string());
        }
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let bindings = self.bindings.clone();
        let arguments = self.arguments.clone();
        let places = self.model_places.clone();
        syn::visit::visit_block(self, block);
        self.bindings = bindings;
        self.arguments = arguments;
        self.model_places = places;
    }

    fn visit_item(&mut self, item: &'ast syn::Item) {
        if let syn::Item::Fn(function) = item {
            let findings = Self::inspect(
                self.context,
                self.ordinal,
                self.module,
                self.file,
                &function.sig,
                &function.block,
                None,
            );
            self.findings.extend(findings);
        }
    }
}
