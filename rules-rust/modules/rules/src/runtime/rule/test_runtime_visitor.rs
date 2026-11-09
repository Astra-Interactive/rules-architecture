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
                if module.tree != SourceTree::Src
                    && module.module.path.iter().any(|segment| segment == "util")
                {
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
                        "tokio::time::sleep" | "tokio::time::interval" | "tokio::time::timeout"
                    )
                })
        {
            self.report(expression.span());
        }
        syn::visit::visit_expr_call(self, expression);
    }
}
