use crate::source::util::parse_macro_arguments;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct LifecycleBodyVisitor<'scope> {
    context: &'scope crate::check::model::RuleContext,
    ordinal: usize,
    module: usize,
    method: &'scope str,
    owned: &'scope [String],
    scopes: &'scope [String],
    task_scopes: &'scope [String],
    shared: &'scope [String],
    pub(crate) invalid: bool,
    pub(crate) aborted: BTreeSet<String>,
    pub(crate) child_lifecycle: bool,
    pub(crate) forwarded: BTreeSet<String>,
    pub(crate) paths: Vec<Vec<String>>,
    flow_cache: RefCell<BTreeMap<String, Vec<Vec<String>>>>,
}

impl LifecycleBodyVisitor<'_> {
    pub(crate) fn inspect<'scope>(
        context: &'scope crate::check::model::RuleContext,
        ordinal: usize,
        module: usize,
        block: &syn::Block,
        method: &'scope str,
        fields: LifecycleFields<'scope>,
    ) -> LifecycleBodyVisitor<'scope> {
        let LifecycleFields {
            owned,
            scopes,
            task_scopes,
            shared,
        } = fields;
        let mut visitor = LifecycleBodyVisitor {
            context,
            ordinal,
            module,
            method,
            owned,
            scopes,
            task_scopes,
            shared,
            invalid: false,
            aborted: BTreeSet::new(),
            child_lifecycle: false,
            forwarded: BTreeSet::new(),
            paths: Vec::new(),
            flow_cache: RefCell::new(BTreeMap::new()),
        };
        visitor.visit_block(block);
        visitor.aborted = scopes
            .iter()
            .filter(|scope| {
                let action = format!("abort:{scope}");
                visitor
                    .block_paths(block, vec![Vec::new()], std::slice::from_ref(&action))
                    .iter()
                    .all(|path| path.contains(&action))
            })
            .cloned()
            .collect();
        visitor.forwarded = owned
            .iter()
            .filter(|child| {
                visitor
                    .block_paths(block, vec![Vec::new()], std::slice::from_ref(child))
                    .iter()
                    .all(|path| path.contains(child))
            })
            .cloned()
            .collect();
        for (position, child) in owned.iter().enumerate() {
            for other in owned.iter().skip(position + 1) {
                visitor.paths.extend(visitor.block_paths(
                    block,
                    vec![Vec::new()],
                    &[child.clone(), other.clone()],
                ));
            }
        }
        visitor
    }

    fn append_action(path: &mut Vec<String>, action: String, tracked: &[String]) {
        if tracked.contains(&action) && !path.contains(&action) {
            path.push(action);
        }
    }

    fn block_paths(
        &self,
        block: &syn::Block,
        mut paths: Vec<Vec<String>>,
        tracked: &[String],
    ) -> Vec<Vec<String>> {
        let key = format!(
            "{:?}|{paths:?}|{tracked:?}",
            crate::source::util::find_node_span(block.span())
        );
        let cached = self.flow_cache.borrow().get(&key).cloned();
        if let Some(cached) = cached {
            return cached;
        }
        for statement in &block.stmts {
            let syn::Stmt::Expr(expression, _) = statement else {
                continue;
            };
            paths = paths
                .into_iter()
                .flat_map(|path| {
                    if path.last().is_some_and(|step| step == "return") {
                        return vec![path];
                    }
                    self.expression_paths(expression, path, tracked)
                })
                .collect();
            paths.sort();
            paths.dedup();
        }
        self.flow_cache.borrow_mut().insert(key, paths.clone());
        paths
    }

    fn expression_paths(
        &self,
        expression: &syn::Expr,
        mut path: Vec<String>,
        tracked: &[String],
    ) -> Vec<Vec<String>> {
        match expression {
            syn::Expr::If(branch) => {
                let literal = match branch.cond.as_ref() {
                    syn::Expr::Lit(value) => match &value.lit {
                        syn::Lit::Bool(value) => Some(value.value),
                        _ => None,
                    },
                    _ => None,
                };
                let mut paths = if literal == Some(false) {
                    Vec::new()
                } else {
                    self.block_paths(&branch.then_branch, vec![path.clone()], tracked)
                };
                if literal != Some(true) {
                    paths.extend(match &branch.else_branch {
                        Some((_, expression)) => self.expression_paths(expression, path, tracked),
                        None => vec![path],
                    });
                }
                paths
            }
            syn::Expr::Match(branch) => branch
                .arms
                .iter()
                .flat_map(|arm| self.expression_paths(&arm.body, path.clone(), tracked))
                .collect(),
            syn::Expr::Block(block) => self.block_paths(&block.block, vec![path], tracked),
            syn::Expr::Return(_) => {
                path.push("return".to_owned());
                vec![path]
            }
            syn::Expr::MethodCall(call) => {
                if let Some(field) = Self::field(&call.receiver) {
                    if call.method == self.method && self.owned.contains(&field) {
                        Self::append_action(&mut path, field, tracked);
                    } else if self.method == "on_disable"
                        && call.method == "abort"
                        && self.scopes.contains(&field)
                    {
                        Self::append_action(&mut path, format!("abort:{field}"), tracked);
                    }
                }
                vec![path]
            }
            syn::Expr::Call(call) => {
                if self.same_ufcs(call)
                    && let Some(field) = call.args.first().and_then(Self::field)
                {
                    Self::append_action(&mut path, field, tracked);
                }
                vec![path]
            }
            _ => vec![path],
        }
    }

    fn field(expression: &syn::Expr) -> Option<String> {
        if let syn::Expr::Reference(reference) = expression {
            return Self::field(&reference.expr);
        }
        let syn::Expr::Field(field) = expression else {
            return None;
        };
        if !matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self")) {
            return None;
        }
        match &field.member {
            syn::Member::Named(name) => Some(name.to_string()),
            _ => None,
        }
    }

    fn same_ufcs(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        if callee
            .path
            .segments
            .last()
            .is_none_or(|segment| segment.ident != self.method)
        {
            return false;
        }
        let mut path = callee.path.clone();
        path.segments.pop();
        path.segments.pop_punct();
        let receiver = call.args.first().and_then(Self::field);
        receiver.is_some_and(|field| self.owned.contains(&field))
            && crate::module::util::lifecycle_trait_path(
                self.context,
                self.ordinal,
                self.module,
                &path,
            )
    }
}

impl LifecycleBodyVisitor<'_> {
    fn is_own_shared_clone(&self, call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        let arc_clone = self
            .context
            .resolve_external_call_paths(self.ordinal, self.module, &callee.path)
            .iter()
            .any(|target| {
                ["std", "alloc"].contains(&target.package.as_str())
                    && target
                        .segments
                        .ends_with(&["Arc".to_owned(), "clone".to_owned()])
            });
        self.method == "on_enable"
            && arc_clone
            && call
                .args
                .first()
                .and_then(Self::field)
                .is_some_and(|field| self.shared.contains(&field))
    }

    fn is_own_task_start(&self, call: &syn::ExprMethodCall) -> bool {
        self.method == "on_enable"
            && call.method == "spawn"
            && Self::field(&call.receiver).is_some_and(|field| self.task_scopes.contains(&field))
    }

    fn is_own_scope_reload(&self, call: &syn::ExprMethodCall) -> bool {
        self.method == "on_reload"
            && call.method == "reload"
            && Self::field(&call.receiver).is_some_and(|field| self.scopes.contains(&field))
    }
}

impl<'ast> Visit<'ast> for LifecycleBodyVisitor<'_> {
    fn visit_expr_await(&mut self, expression: &'ast syn::ExprAwait) {
        self.invalid |= self.method == "on_disable";
        syn::visit::visit_expr_await(self, expression);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let child = self.same_ufcs(call);
        self.invalid |= !child && !self.is_own_shared_clone(call);
        self.child_lifecycle |= child;
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if self.is_own_task_start(call) {
            return;
        }
        if self.is_own_scope_reload(call) {
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        let name = call.method.to_string();
        let field = Self::field(&call.receiver);
        let shared_clone = self.method == "on_enable"
            && name == "clone"
            && field
                .as_ref()
                .is_some_and(|field| self.shared.contains(field));
        let abort = self.method == "on_disable"
            && name == "abort"
            && field
                .as_ref()
                .is_some_and(|field| self.scopes.contains(field));
        let child = name == self.method
            && field
                .as_ref()
                .is_some_and(|field| self.owned.contains(field));
        if abort {
            self.aborted.extend(field);
        }
        self.child_lifecycle |= child;
        self.invalid |= !abort && !child && !shared_clone;
        syn::visit::visit_expr_method_call(self, call);
    }
}

pub(crate) struct LifecycleFields<'scope> {
    pub(crate) owned: &'scope [String],
    pub(crate) scopes: &'scope [String],
    pub(crate) task_scopes: &'scope [String],
    pub(crate) shared: &'scope [String],
}
