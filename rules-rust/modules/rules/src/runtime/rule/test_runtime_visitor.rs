use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::util::node_predicate;
use crate::source::model::{SourceFile, SourceTree};
use crate::source::util::find_node_span;
use crate::workspace::model::DependencyKind;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TestRuntimeVisitor<'source> {
    context: &'source RuleContext,
    ordinal: usize,
    module: usize,
    file: &'source SourceFile,
    rule: &'static str,
    findings: Vec<RuleFinding>,
    test_without_pause: bool,
}

impl TestRuntimeVisitor<'_> {
    fn builtin_assertion(&self, item: &syn::Macro) -> bool {
        let Some(name) = item.path.get_ident() else {
            return false;
        };
        if !["assert", "assert_eq", "assert_ne"]
            .iter()
            .any(|expected| name == expected)
        {
            return false;
        }
        let index = &self.context.indexes[self.ordinal];
        !index.items.iter().any(|item| *name == item.name)
            && !index
                .imports
                .iter()
                .any(|import| import.name.as_deref() == Some(name.to_string().as_str()))
    }
    fn model_parameter(&self, module: usize, parameter: &syn::FnArg) -> bool {
        let syn::FnArg::Typed(parameter) = parameter else {
            return false;
        };
        self.context
            .resolve_workspace_type(self.ordinal, module, &parameter.ty)
            .iter()
            .any(|target| {
                target.path.item.is_some_and(|identity| {
                    let index = &self.context.indexes[target.index];
                    index.modules[index.items[identity].module]
                        .module
                        .path
                        .iter()
                        .any(|segment| segment == "model")
                })
            })
    }

    fn model_rule(&self, item: &crate::source::model::IndexedItem) -> bool {
        let index = &self.context.indexes[self.ordinal];
        if !index.modules[item.module]
            .module
            .path
            .iter()
            .any(|segment| segment == "util")
        {
            return false;
        }
        let syn::Item::Fn(function) = &item.syntax else {
            return false;
        };
        let body = syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: (*function.block).clone(),
        });
        let effects = crate::runtime::rule::ClosureEffects::collect(&body);
        if !effects.calls.is_empty()
            || !effects.methods.is_empty()
            || effects.assigns_capture
            || !effects.macros.is_empty()
        {
            return false;
        }
        function.sig.asyncness.is_none()
            && function
                .sig
                .inputs
                .iter()
                .any(|parameter| self.model_parameter(item.module, parameter))
    }

    fn model_targets(
        &self,
        expression: &syn::Expr,
        bindings: &[ModelBinding],
    ) -> Vec<crate::source::model::WorkspaceResolvedPath> {
        match expression {
            syn::Expr::Reference(reference) => self.model_targets(&reference.expr, bindings),
            syn::Expr::Paren(paren) => self.model_targets(&paren.expr, bindings),
            syn::Expr::Path(path) => bindings
                .iter()
                .rev()
                .find(|binding| path.path.is_ident(&binding.name))
                .map(|binding| binding.targets.clone())
                .unwrap_or_else(|| {
                    self.context
                        .resolve_workspace_path(self.ordinal, self.module, &path.path)
                }),
            syn::Expr::Struct(structure) => {
                self.context
                    .resolve_workspace_path(self.ordinal, self.module, &structure.path)
            }
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = &*call.func else {
                    return Vec::new();
                };
                let mut owner = path.path.clone();
                owner.segments.pop();
                owner.segments.pop_punct();
                self.context
                    .resolve_workspace_path(self.ordinal, self.module, &owner)
            }
            _ => Vec::new(),
        }
    }

    fn rooted_in_self(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(path) => path.path.is_ident("self"),
            syn::Expr::Field(field) => Self::rooted_in_self(&field.base),
            syn::Expr::MethodCall(call) => Self::rooted_in_self(&call.receiver),
            syn::Expr::Paren(paren) => Self::rooted_in_self(&paren.expr),
            syn::Expr::Reference(reference) => Self::rooted_in_self(&reference.expr),
            _ => false,
        }
    }

    fn is_data_method(call: &syn::ExprMethodCall) -> bool {
        !matches!(
            call.method.to_string().as_str(),
            "spawn"
                | "block_on"
                | "send"
                | "recv"
                | "lock"
                | "read"
                | "write"
                | "join"
                | "sleep"
                | "wait"
        ) && !call.method.to_string().starts_with("blocking_")
            && Self::rooted_in_self(&call.receiver)
    }

    fn pure_method(item: &syn::ImplItem, name: &syn::Ident) -> bool {
        let syn::ImplItem::Fn(function) = item else {
            return false;
        };
        if function.sig.ident != *name || function.sig.asyncness.is_some() {
            return false;
        }
        if !crate::source::model::CfgPredicate::from_attributes(&function.attrs).is_satisfiable() {
            return false;
        }
        let body = syn::Expr::Block(syn::ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: function.block.clone(),
        });
        let effects = crate::runtime::rule::ClosureEffects::collect(&body);
        effects.calls.is_empty()
            && effects.methods.iter().all(Self::is_data_method)
            && effects.macros.is_empty()
            && !effects.assigns_capture
    }

    fn pure_model_method(
        &self,
        targets: &[crate::source::model::WorkspaceResolvedPath],
        name: &syn::Ident,
    ) -> bool {
        !targets.is_empty() && targets.iter().all(|target| {
            let Some(identity) = target.path.item else {
                return false;
            };
            let index = &self.context.indexes[target.index];
            let item = &index.items[identity];
            if !index.modules[item.module]
                .module
                .path
                .iter()
                .any(|segment| segment == "model")
            {
                return false;
            }
            let methods: Vec<_> = index
                .implementations
                .iter()
                .filter(|implementation| {
                    implementation.cfg.and(&target.path.cfg).is_satisfiable()
                        && index
                            .resolve_type_items(
                                implementation.module,
                                &implementation.syntax.self_ty,
                            )
                            .contains(&identity)
                })
                .flat_map(|implementation| {
                    implementation
                        .syntax
                        .items
                        .iter()
                        .filter(move |item| matches!(item, syn::ImplItem::Fn(function)
                            if function.sig.ident == *name
                                && implementation.cfg.and(&target.path.cfg)
                                    .and(&crate::source::model::CfgPredicate::from_attributes(&function.attrs)).is_satisfiable()))
                }).collect();
            !methods.is_empty() && methods.iter().all(|item| Self::pure_method(item, name))
        })
    }

    fn model_call(&self, call: &syn::ExprCall, bindings: &[ModelBinding]) -> bool {
        let syn::Expr::Path(path) = &*call.func else {
            return false;
        };
        if bindings
            .iter()
            .rev()
            .any(|binding| path.path.is_ident(&binding.name))
        {
            return false;
        }
        let model_rule = self
            .context
            .resolve_workspace_path(self.ordinal, self.module, &path.path)
            .iter()
            .any(|target| {
                target.index == self.ordinal
                    && target.path.item.is_some_and(|identity| {
                        self.model_rule(&self.context.indexes[target.index].items[identity])
                    })
            });
        let expression = syn::Expr::Call(call.clone());
        let constructor = path.path.segments.last().is_some_and(|name| {
            self.pure_model_method(&self.model_targets(&expression, bindings), &name.ident)
        });
        model_rule || constructor
    }

    fn check_model_if(
        &self,
        conditional: &syn::ExprIf,
        bindings: &mut Vec<ModelBinding>,
        exercised: &mut bool,
    ) -> bool {
        self.model_expression(&conditional.cond, bindings, exercised)
            && self.model_block(&conditional.then_branch, bindings, exercised)
            && conditional
                .else_branch
                .as_ref()
                .is_none_or(|(_, otherwise)| self.model_expression(otherwise, bindings, exercised))
    }

    fn check_model_match(
        &self,
        matched: &syn::ExprMatch,
        bindings: &mut Vec<ModelBinding>,
        exercised: &mut bool,
    ) -> bool {
        if !self.model_expression(&matched.expr, bindings, exercised) {
            return false;
        }
        matched.arms.iter().all(|arm| {
            let length = bindings.len();
            let pattern = match &arm.pat {
                syn::Pat::Guard(guard) => &*guard.pat,
                pattern => pattern,
            };
            match pattern {
                syn::Pat::Ident(name) if name.subpat.is_none() => bindings.push(ModelBinding {
                    name: name.ident.to_string(),
                    targets: Vec::new(),
                }),
                syn::Pat::Wild(_) | syn::Pat::Lit(_) | syn::Pat::Path(_) | syn::Pat::Range(_) => {}
                _ => return false,
            }
            let guard = match &arm.pat {
                syn::Pat::Guard(guard) => self.model_expression(&guard.guard, bindings, exercised),
                _ => true,
            };
            let pure = guard && self.model_expression(&arm.body, bindings, exercised);
            bindings.truncate(length);
            pure
        })
    }

    fn model_expression(
        &self,
        expression: &syn::Expr,
        bindings: &mut Vec<ModelBinding>,
        exercised: &mut bool,
    ) -> bool {
        match expression {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => true,
            syn::Expr::Reference(reference) => {
                self.model_expression(&reference.expr, bindings, exercised)
            }
            syn::Expr::Paren(paren) => self.model_expression(&paren.expr, bindings, exercised),
            syn::Expr::Field(field) => self.model_expression(&field.base, bindings, exercised),
            syn::Expr::Unary(unary) => self.model_expression(&unary.expr, bindings, exercised),
            syn::Expr::Binary(binary) => {
                self.model_expression(&binary.left, bindings, exercised)
                    && self.model_expression(&binary.right, bindings, exercised)
            }
            syn::Expr::Struct(structure) => structure
                .fields
                .iter()
                .all(|field| self.model_expression(&field.expr, bindings, exercised)),
            syn::Expr::Block(block) => self.model_block(&block.block, bindings, exercised),
            syn::Expr::If(conditional) => self.check_model_if(conditional, bindings, exercised),
            syn::Expr::Match(matched) => self.check_model_match(matched, bindings, exercised),
            syn::Expr::MethodCall(call) => {
                let targets = self.model_targets(&call.receiver, bindings);
                if !self.pure_model_method(&targets, &call.method) {
                    return false;
                }
                *exercised = true;
                self.model_expression(&call.receiver, bindings, exercised)
                    && call
                        .args
                        .iter()
                        .all(|argument| self.model_expression(argument, bindings, exercised))
            }
            syn::Expr::Call(call) => {
                if !self.model_call(call, bindings) {
                    return false;
                }
                *exercised = true;
                call.args
                    .iter()
                    .all(|argument| self.model_expression(argument, bindings, exercised))
            }
            syn::Expr::Macro(item) => self.model_assertion(&item.mac, bindings, exercised),
            _ => false,
        }
    }

    fn model_assertion(
        &self,
        item: &syn::Macro,
        bindings: &mut Vec<ModelBinding>,
        exercised: &mut bool,
    ) -> bool {
        if !self.builtin_assertion(item) {
            return false;
        }
        crate::source::util::parse_macro_arguments(item)
            .iter()
            .all(|argument| self.model_expression(argument, bindings, exercised))
    }

    fn model_block(
        &self,
        block: &syn::Block,
        bindings: &mut Vec<ModelBinding>,
        exercised: &mut bool,
    ) -> bool {
        let length = bindings.len();
        let pure = block.stmts.iter().all(|statement| match statement {
            syn::Stmt::Local(local) => {
                let Some(init) = &local.init else {
                    return false;
                };
                if !self.model_expression(&init.expr, bindings, exercised) {
                    return false;
                }
                let (pattern, targets) = match &local.pat {
                    syn::Pat::Type(typed) => (
                        &*typed.pat,
                        self.context
                            .resolve_workspace_type(self.ordinal, self.module, &typed.ty),
                    ),
                    pattern => (pattern, self.model_targets(&init.expr, bindings)),
                };
                let syn::Pat::Ident(name) = pattern else {
                    return false;
                };
                bindings.push(ModelBinding {
                    name: name.ident.to_string(),
                    targets,
                });
                true
            }
            syn::Stmt::Expr(expression, _) => {
                self.model_expression(expression, bindings, exercised)
            }
            syn::Stmt::Macro(item) => self.model_assertion(&item.mac, bindings, exercised),
            syn::Stmt::Item(_) => false,
        });
        bindings.truncate(length);
        pure
    }

    fn model_only(&self, block: &syn::Block) -> bool {
        let mut exercised = false;
        self.model_block(block, &mut Vec::new(), &mut exercised) && exercised
    }
    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                if file.is_generated {
                    continue;
                }
                let Some((ordinal, index)) =
                    context.indexes.iter().enumerate().find(|(_, index)| {
                        index.crate_name == file.module.crate_name
                            && index.target == file.module.target
                    })
                else {
                    continue;
                };
                let Some(module) = index
                    .modules
                    .iter()
                    .find(|module| module.module == file.module)
                else {
                    continue;
                };
                let mut visitor = TestRuntimeVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    file,
                    rule,
                    findings: Vec::new(),
                    test_without_pause: false,
                };
                visitor.visit_file(&file.syntax);
                findings.extend(visitor.findings);
            }
        }
        findings
    }

    fn report(&mut self, span: proc_macro2::Span) {
        let index = &self.context.indexes[self.ordinal];
        if !node_predicate(index, self.module, &self.file.path, find_node_span(span))
            .is_satisfiable()
        {
            return;
        }
        self.findings.push(build_node_finding(self.rule, &self.file.path, find_node_span(span),
            format_module_path(&index.crate_name, &index.modules[self.module].module.path),
            "Test model rules synchronously; async time tests use the paused current-thread runtime and dev test-util (testing.rs.md)".to_owned()));
    }

    fn tokio_test(&self, attribute: &syn::Attribute) -> bool {
        self.context
            .resolve_external_call_paths(self.ordinal, self.module, attribute.path())
            .iter()
            .any(|path| path.format_path() == "tokio::test")
    }

    fn test_options(attribute: &syn::Attribute) -> (bool, bool) {
        let Ok(options) = attribute.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        ) else {
            return (false, false);
        };
        let mut paused = false;
        let mut multi_thread = false;
        for option in options {
            let syn::Meta::NameValue(value) = option else {
                continue;
            };
            match &value.value {
                syn::Expr::Lit(literal) if value.path.is_ident("start_paused") => {
                    paused = matches!(&literal.lit, syn::Lit::Bool(value) if value.value)
                }
                syn::Expr::Lit(literal) if value.path.is_ident("flavor") => {
                    multi_thread = matches!(&literal.lit, syn::Lit::Str(value) if value.value() == "multi_thread")
                }
                _ => {}
            }
        }
        (paused, multi_thread)
    }

    fn has_test_util(&self) -> bool {
        let index = &self.context.indexes[self.ordinal];
        self.context
            .workspace
            .members
            .iter()
            .filter(|member| member.name == index.crate_name)
            .any(|member| {
                member.dependencies.iter().any(|dependency| {
                    dependency.name == "tokio"
                        && dependency.kind == DependencyKind::Development
                        && dependency
                            .features
                            .iter()
                            .any(|feature| feature == "test-util")
                })
            })
    }
}

impl<'ast> Visit<'ast> for TestRuntimeVisitor<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &crate::source::util::parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let parent = self.module;
        if let Some(module) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|module| {
                module.parent == Some(parent) && module.span == find_node_span(item.span())
            })
        {
            self.module = module.id;
        }
        syn::visit::visit_item_mod(self, item);
        self.module = parent;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let parent = self.test_without_pause;
        self.test_without_pause = false;
        for attribute in &item.attrs {
            let tokio = self.tokio_test(attribute);
            let builtin = attribute.path().is_ident("test");
            if self.rule == "ModelRuleTest" && tokio {
                let module = &self.context.indexes[self.ordinal].modules[self.module];
                if module.tree != SourceTree::Src && self.model_only(&item.block) {
                    self.report(attribute.span());
                }
            }
            if self.rule != "TestTimePaused" || (!tokio && !builtin) {
                continue;
            }
            let (paused, multi_thread) = Self::test_options(attribute);
            self.test_without_pause = !paused;
            if paused && (multi_thread || !self.has_test_util()) {
                self.report(attribute.span());
            }
        }
        syn::visit::visit_item_fn(self, item);
        self.test_without_pause = parent;
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if self.rule == "TestTimePaused"
            && self.test_without_pause
            && let syn::Expr::Path(path) = &*expression.func
            && self
                .context
                .resolve_external_call_paths(self.ordinal, self.module, &path.path)
                .iter()
                .any(|path| {
                    matches!(
                        path.format_path().as_str(),
                        "tokio::time::sleep"
                            | "tokio::time::sleep_until"
                            | "tokio::time::interval"
                            | "tokio::time::interval_at"
                            | "tokio::time::timeout"
                            | "tokio::time::timeout_at"
                    )
                })
        {
            self.report(expression.span());
        }
        syn::visit::visit_expr_call(self, expression);
    }
}

struct ModelBinding {
    name: String,
    targets: Vec<crate::source::model::WorkspaceResolvedPath>,
}
