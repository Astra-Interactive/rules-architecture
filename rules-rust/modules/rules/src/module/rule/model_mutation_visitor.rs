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
    model_places: BTreeSet<String>,
    findings: Vec<RuleFinding>,
}

impl<'context> ModelMutationVisitor<'context> {
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
            model_places: BTreeSet::new(),
            findings: Vec::new(),
        };
        if let Some(typed) = self_type {
            visitor.bindings.insert(
                "self".to_owned(),
                context.resolve_workspace_type(ordinal, module, typed),
            );
        }
        for parameter in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = parameter {
                visitor.bind_pattern(
                    &parameter.pat,
                    context.resolve_workspace_type(ordinal, module, &parameter.ty),
                );
            }
        }
        visitor.visit_block(block);
        visitor.findings
    }

    fn bind_pattern(&mut self, pattern: &syn::Pat, targets: Vec<WorkspaceResolvedPath>) {
        match pattern {
            syn::Pat::Ident(binding) => {
                self.model_places.remove(&binding.ident.to_string());
                self.bindings.insert(binding.ident.to_string(), targets);
                if let Some((_, nested)) = &binding.subpat {
                    self.bind_pattern(nested, Vec::new());
                }
            }
            syn::Pat::Type(typed) => self.bind_pattern(&typed.pat, targets),
            syn::Pat::Tuple(tuple) => {
                for element in &tuple.elems {
                    self.bind_pattern(element, Vec::new());
                }
            }
            syn::Pat::TupleStruct(tuple) => {
                for element in &tuple.elems {
                    self.bind_pattern(element, Vec::new());
                }
            }
            syn::Pat::Struct(structure) => {
                for field in &structure.fields {
                    self.bind_pattern(&field.pat, Vec::new());
                }
            }
            syn::Pat::Slice(slice) => {
                for element in &slice.elems {
                    self.bind_pattern(element, Vec::new());
                }
            }
            syn::Pat::Reference(reference) => self.bind_pattern(&reference.pat, targets),
            syn::Pat::Paren(paren) => self.bind_pattern(&paren.pat, targets),
            syn::Pat::Guard(guard) => self.bind_pattern(&guard.pat, targets),
            syn::Pat::Or(alternatives) => {
                for alternative in &alternatives.cases {
                    self.bind_pattern(alternative, Vec::new());
                }
            }
            _ => {}
        }
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
            let field = match &field.member {
                syn::Member::Named(name) => structure
                    .fields
                    .iter()
                    .find(|field| field.ident.as_ref() == Some(name)),
                syn::Member::Unnamed(position) => {
                    structure.fields.iter().nth(position.index as usize)
                }
            };
            if let Some(field) = field {
                targets.extend(self.context.resolve_workspace_type(
                    owner.index,
                    item.module,
                    &field.ty,
                ));
            }
        }
        targets
    }

    fn targets(&self, expression: &syn::Expr) -> Vec<WorkspaceResolvedPath> {
        match expression {
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
            syn::Expr::Field(field) => self.targets(&field.base).iter().any(|target| {
                target.path.item.is_some_and(|identity| {
                    is_model_item(
                        self.context,
                        target.index,
                        &self.context.indexes[target.index].items[identity],
                    )
                })
            }),
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

    fn report_write(&mut self, target: &syn::Expr, span: proc_macro2::Span) {
        if self.model_place(target) {
            self.findings.push(build_node_finding(
                "ModelShouldBeImmutable",
                self.file,
                find_node_span(span),
                "model field".to_owned(),
                "a model transition returns a new value instead of assigning a field".to_owned(),
            ));
        }
    }
}

impl<'ast> Visit<'ast> for ModelMutationVisitor<'_> {
    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        let bindings = self.bindings.clone();
        let places = self.model_places.clone();
        self.bind_pattern(&expression.pat, Vec::new());
        self.visit_block(&expression.body);
        self.bindings = bindings;
        self.model_places = places;
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        self.visit_expr(&expression.expr);
        for arm in &expression.arms {
            let bindings = self.bindings.clone();
            let places = self.model_places.clone();
            self.bind_pattern(&arm.pat, self.targets(&expression.expr));
            self.visit_expr(&arm.body);
            self.bindings = bindings;
            self.model_places = places;
        }
    }

    fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
        self.visit_expr(&expression.expr);
        self.bind_pattern(&expression.pat, self.targets(&expression.expr));
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        let bindings = self.bindings.clone();
        let places = self.model_places.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.then_branch);
        self.bindings = bindings;
        self.model_places = places;
        if let Some((_, expression)) = &expression.else_branch {
            self.visit_expr(expression);
        }
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        let bindings = self.bindings.clone();
        let places = self.model_places.clone();
        self.visit_expr(&expression.cond);
        self.visit_block(&expression.body);
        self.bindings = bindings;
        self.model_places = places;
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let bindings = self.bindings.clone();
        let places = self.model_places.clone();
        for parameter in &closure.inputs {
            let targets = match parameter {
                syn::Pat::Type(typed) => {
                    self.context
                        .resolve_workspace_type(self.ordinal, self.module, &typed.ty)
                }
                _ => Vec::new(),
            };
            self.bind_pattern(parameter, targets);
        }
        self.visit_expr(&closure.body);
        self.bindings = bindings;
        self.model_places = places;
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
            syn::Pat::Type(typed) => {
                self.context
                    .resolve_workspace_type(self.ordinal, self.module, &typed.ty)
            }
            _ => local
                .init
                .as_ref()
                .map(|initializer| self.targets(&initializer.expr))
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
        self.bind_pattern(&local.pat, targets);
        if let syn::Pat::Ident(binding) = pattern
            && place
        {
            self.model_places.insert(binding.ident.to_string());
        }
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let bindings = self.bindings.clone();
        let places = self.model_places.clone();
        syn::visit::visit_block(self, block);
        self.bindings = bindings;
        self.model_places = places;
    }

    fn visit_item(&mut self, _: &'ast syn::Item) {}
}
