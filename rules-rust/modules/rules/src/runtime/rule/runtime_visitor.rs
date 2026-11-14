use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::config::model::Role;
use crate::runtime::util::node_predicate;
use crate::source::model::CfgPredicate;
use crate::source::model::CrateIndex;
use crate::source::model::Resolution;
use crate::source::model::SourceFile;
use crate::source::model::SourceTree;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct RuntimeVisitor<'source> {
    context: &'source RuleContext,
    index: &'source CrateIndex,
    file: &'source SourceFile,
    module: usize,
    ordinal: usize,
    rule: &'static str,
    is_main: bool,
    findings: Vec<RuleFinding>,
}

impl RuntimeVisitor<'_> {
    fn report(&mut self, span: proc_macro2::Span, message: &str) {
        self.findings.push(build_node_finding(
            self.rule,
            &self.file.path,
            find_node_span(span),
            format_module_path(
                &self.index.crate_name,
                &self.index.modules[self.module].module.path,
            ),
            message.to_owned(),
        ));
    }

    fn is_test(&self) -> bool {
        self.index.modules[self.module].tree != SourceTree::Src
    }

    fn is_technology(&self) -> bool {
        self.context
            .project_config
            .crates
            .iter()
            .find(|config| config.name == self.index.crate_name)
            .is_some_and(|config| {
                config.role == Some(Role::Technology)
                    || self.index.modules[self.module]
                        .module
                        .path
                        .iter()
                        .any(|segment| config.technology_words.contains(segment))
            })
    }

    fn check_runtime_call(&mut self, span: proc_macro2::Span, is_block_on: bool) {
        if self.rule == "BlockOnInTest" && self.is_test() {
            self.report(span, Self::BLOCK_ON_IN_TEST_MESSAGE);
        }
        if self.rule != "BlockOnOutsideBoundary" || self.is_main {
            return;
        }
        if !is_block_on || !self.is_technology() {
            self.report(span, Self::BLOCK_ON_OUTSIDE_BOUNDARY_MESSAGE);
        }
    }

    fn is_dependency_block_on(resolved: &str) -> bool {
        let mut segments = resolved.rsplit("::");
        segments.next() == Some("block_on")
            && segments
                .next()
                .and_then(|owner| owner.chars().next())
                .is_some_and(char::is_lowercase)
    }

    fn resolved_call(&self, module: usize, path: &syn::Path, depth: usize) -> Option<String> {
        if depth > 8 {
            return None;
        }
        match self.index.resolve_path(module, path) {
            Resolution::External(path) => Some(path),
            Resolution::Internal(targets) => targets.iter().find_map(|target| {
                let item = target
                    .item
                    .and_then(|identity| self.index.items.get(identity))?;
                let syn::Item::Type(alias) = &item.syntax else {
                    return None;
                };
                let syn::Type::Path(aliased) = &*alias.ty else {
                    return None;
                };
                let mut aliased_path = aliased.path.clone();
                if path.segments.len() > 1 {
                    aliased_path.segments.push(path.segments.last()?.clone());
                }
                self.resolved_call(item.module, &aliased_path, depth + 1)
            }),
            _ => None,
        }
    }

    fn has_active_ignore(&self, meta: &syn::Meta, active: &CfgPredicate) -> bool {
        if meta.path().is_ident("ignore") {
            return true;
        }
        let syn::Meta::List(list) = meta else {
            return false;
        };
        if !list.path.is_ident("cfg_attr") {
            return false;
        }
        let Ok(arguments) =
            list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
        else {
            return false;
        };
        let mut arguments = arguments.iter();
        let Some(condition) = arguments.next() else {
            return false;
        };
        let attribute: syn::Attribute = syn::parse_quote!(#[cfg(#condition)]);
        let predicate = CfgPredicate::from_attributes(&[attribute]);
        let active = active.in_scope(self.ordinal);
        let predicate = predicate.in_scope(self.ordinal);
        if !active.is_satisfiable() || !active.implies(&predicate) {
            return false;
        }
        arguments.any(|nested| self.has_active_ignore(nested, &active))
    }

    fn has_tokio_main(&self, meta: &syn::Meta) -> bool {
        let path = meta.path();
        if self.resolved_call(self.module, path, 0).as_deref() == Some("tokio::main") {
            return true;
        }
        let segments = &path.segments;
        if segments.len() == 2 && segments[0].ident == "tokio" && segments[1].ident == "main" {
            return true;
        }
        let syn::Meta::List(list) = meta else {
            return false;
        };
        if !path.is_ident("cfg_attr") {
            return false;
        }
        let Ok(arguments) =
            list.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
        else {
            return false;
        };
        arguments
            .iter()
            .skip(1)
            .any(|nested| self.has_tokio_main(nested))
    }

    pub(crate) fn check_context(context: &RuleContext, rule: &'static str) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
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
                let mut visitor = RuntimeVisitor {
                    context,
                    index,
                    file,
                    module: module.id,
                    ordinal,
                    rule,
                    is_main: false,
                    findings: Vec::new(),
                };
                visitor.visit_file(&file.syntax);
                findings.extend(visitor.findings);
            }
        }
        if matches!(rule, "BlockOnInTest" | "BlockOnOutsideBoundary") {
            findings.extend(crate::runtime::rule::TypedRuntimeVisitor::check_context(
                context, rule,
            ));
        }
        findings
    }

    pub(crate) const BLOCK_ON_IN_TEST_MESSAGE: &'static str = "Run async tests with tokio::test instead of building or blocking a runtime (testing.rs.md)";

    pub(crate) const BLOCK_ON_OUTSIDE_BOUNDARY_MESSAGE: &'static str = "Build and block a runtime only in entry-point root main; technology boundaries may block_on (di-manual.rs.md)";
}

impl<'ast> Visit<'ast> for RuntimeVisitor<'_> {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let parent = self.module;
        if let Some(module) = self.index.modules.iter().find(|module| {
            module.parent == Some(parent)
                && module.span == find_node_span(item.span())
                && module
                    .module
                    .path
                    .last()
                    .is_some_and(|name| item.ident == name)
        }) {
            self.module = module.id;
        }
        syn::visit::visit_item_mod(self, item);
        self.module = parent;
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let parent = self.is_main;
        let is_entry = self.context.project_config.crates.iter().any(|config| {
            config.name == self.index.crate_name && config.role == Some(Role::EntryPoint)
        });
        self.is_main = is_entry
            && self.index.modules[self.module].module.path.is_empty()
            && item.sig.ident == "main";
        syn::visit::visit_item_fn(self, item);
        self.is_main = parent;
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if self.rule == "IgnoredTest" {
            let active = node_predicate(
                self.index,
                self.module,
                &self.file.path,
                find_node_span(attribute.span()),
            );
            if self.has_active_ignore(&attribute.meta, &active) {
                self.report(attribute.span(), "Keep tests enabled (no-workarounds.md)");
            }
        }
        let tokio_main = self.has_tokio_main(&attribute.meta);
        if self.rule == "BlockOnOutsideBoundary" && tokio_main {
            self.report(attribute.span(), "Build the async runtime in entry-point main instead of tokio::main (di-manual.rs.md)");
        }
        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_expr_call(&mut self, expression: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*expression.func
            && let Some(resolved) = self.resolved_call(self.module, &path.path, 0)
        {
            let is_block_on = Self::is_dependency_block_on(&resolved)
                || resolved.ends_with("::runtime::Runtime::block_on")
                || resolved.ends_with("::runtime::Handle::block_on");
            let is_runtime_new = resolved == "tokio::runtime::Runtime::new";
            let is_builder_new = resolved.starts_with("tokio::runtime::Builder::new_");
            if is_block_on || is_runtime_new || is_builder_new {
                self.check_runtime_call(expression.span(), is_block_on);
            }
            let is_test_executor =
                resolved.ends_with("::block_on_stream") || resolved.ends_with("::LocalPool::new");
            if is_test_executor && self.rule == "BlockOnInTest" && self.is_test() {
                self.report(expression.span(), Self::BLOCK_ON_IN_TEST_MESSAGE);
            }
        }
        syn::visit::visit_expr_call(self, expression);
    }
}
