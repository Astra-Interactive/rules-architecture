use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::source::model::Resolution;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct GlobalStateVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    file: String,
    is_main: bool,
    findings: Vec<RuleFinding>,
}

impl GlobalStateVisitor<'_> {
    fn report(&mut self, span: proc_macro2::Span, message: &str) {
        self.findings.push(build_node_finding(
            "GlobalState",
            &self.file,
            find_node_span(span),
            self.context.indexes[self.ordinal].modules[self.module]
                .module
                .path
                .join("::"),
            message.to_owned(),
        ));
    }

    fn resolved_package(&self, path: &syn::Path) -> Option<String> {
        let Resolution::External(path) =
            self.context.indexes[self.ordinal].resolve_path(self.module, path)
        else {
            return None;
        };
        Some(
            self.context
                .resolve_external_identity(self.ordinal, &path)
                .package,
        )
    }

    fn is_subscriber_builder(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::MethodCall(call) => self.is_subscriber_builder(&call.receiver),
            syn::Expr::Paren(paren) => self.is_subscriber_builder(&paren.expr),
            syn::Expr::Call(call) => {
                matches!(call.func.as_ref(), syn::Expr::Path(path) if self.resolved_package(&path.path).as_deref() == Some("tracing_subscriber"))
            }
            _ => false,
        }
    }

    pub(crate) fn check(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut visitor = GlobalStateVisitor {
            context,
            ordinal,
            module: 0,
            file: String::new(),
            is_main: false,
            findings: Vec::new(),
        };
        for item in &index.items {
            visitor.module = item.module;
            visitor.file = item.file.clone();
            visitor.is_main = item.name == "main"
                && item.module == 0
                && context
                    .find_role_config(&index.crate_name)
                    .and_then(|config| config.role)
                    == Some(Role::EntryPoint);
            visitor.visit_item(&item.syntax);
        }
        for implementation in &index.implementations {
            visitor.module = implementation.module;
            visitor.file = implementation.file.clone();
            visitor.is_main = false;
            visitor.visit_item_impl(&implementation.syntax);
        }
        visitor.findings
    }
}

impl<'ast> Visit<'ast> for GlobalStateVisitor<'_> {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        let main = self.is_main;
        self.is_main &= function.sig.ident == "main";
        syn::visit::visit_item_fn(self, function);
        self.is_main = main;
    }
    fn visit_macro(&mut self, expression: &'ast syn::Macro) {
        if self.resolved_package(&expression.path).as_deref() == Some("lazy_static") {
            self.report(
                expression.span(),
                "lazy_static state is owned by DI instead of a singleton",
            );
        }
        for argument in &crate::source::util::parse_macro_arguments(expression) {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if self.is_main {
            syn::visit::visit_expr_call(self, call);
            return;
        }
        if let syn::Expr::Path(path) = call.func.as_ref() {
            let Resolution::External(path) =
                self.context.indexes[self.ordinal].resolve_path(self.module, &path.path)
            else {
                syn::visit::visit_expr_call(self, call);
                return;
            };
            let path = self.context.resolve_external_identity(self.ordinal, &path);
            let global = path
                .segments
                .last()
                .is_some_and(|segment| segment == "set_global_default");
            if global && ["tracing", "tracing_core"].contains(&path.package.as_str()) {
                self.report(
                    call.span(),
                    "install the logging subscriber in entry-point main",
                );
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if !self.is_main && call.method == "init" && self.is_subscriber_builder(&call.receiver) {
            self.report(
                call.span(),
                "install the logging subscriber in entry-point main",
            );
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}
