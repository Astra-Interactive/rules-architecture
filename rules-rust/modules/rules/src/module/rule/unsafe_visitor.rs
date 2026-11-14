use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::find_kind_name;
use crate::source::model::{Resolution, SourceFile, SourceTree};
use crate::source::util::{find_node_span, parse_macro_arguments};
use syn::spanned::Spanned;
use syn::visit::Visit;

const POINTER_FUNCTIONS: [&str; 3] = ["null", "null_mut", "into_raw"];
const POINTER_METHODS: [&str; 3] = ["as_ptr", "as_mut_ptr", "into_raw"];

pub(crate) struct UnsafeVisitor<'source> {
    context: &'source RuleContext,
    ordinal: usize,
    module: usize,
    file: &'source SourceFile,
    findings: Vec<RuleFinding>,
}

impl UnsafeVisitor<'_> {
    fn check_span(&mut self, span: proc_macro2::Span) {
        let index = &self.context.indexes[self.ordinal];
        let kind = find_kind_name(self.context, self.ordinal, self.module);
        let technology_unit = kind.is_empty()
            && self
                .context
                .has_technology_implementation_role(&index.crate_name);
        if technology_unit || self.context.is_technology_kind(&index.crate_name, kind) {
            return;
        }
        self.findings.push(build_node_finding(
            "UnsafePlacement",
            &self.file.path,
            find_node_span(span),
            self.file.path.clone(),
            "Keep unsafe code behind a safe technology implementation (arch-package.rs.md)"
                .to_owned(),
        ));
    }

    fn is_entry_point(&self) -> bool {
        let index = &self.context.indexes[self.ordinal];
        self.context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role)
            == Some(crate::config::model::Role::EntryPoint)
    }

    fn exports_symbol(attribute: &syn::Attribute) -> bool {
        let is_symbol =
            |path: &syn::Path| path.is_ident("no_mangle") || path.is_ident("export_name");
        match &attribute.meta {
            syn::Meta::List(list) if list.path.is_ident("unsafe") => list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|metas| metas.iter().any(|meta| is_symbol(meta.path()))),
            meta => is_symbol(meta.path()),
        }
    }

    fn is_host_function(item: &syn::ItemFn) -> bool {
        item.sig.abi.is_some() && item.attrs.iter().any(Self::exports_symbol)
    }

    fn is_pointer_function_call(call: &syn::ExprCall) -> bool {
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        let Some(segment) = callee.path.segments.last() else {
            return false;
        };
        let is_producer = POINTER_FUNCTIONS.iter().any(|name| segment.ident == *name);
        is_producer && call.args.iter().all(Self::is_safe_argument)
    }

    fn is_pointer_method_call(call: &syn::ExprMethodCall) -> bool {
        let is_producer = POINTER_METHODS.iter().any(|name| call.method == *name);
        is_producer
            && Self::is_safe_argument(&call.receiver)
            && call.args.iter().all(Self::is_safe_argument)
    }

    fn is_safe_argument(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Path(_) | syn::Expr::Lit(_) => true,
            syn::Expr::Cast(cast) => Self::is_safe_argument(&cast.expr),
            syn::Expr::Reference(reference) => Self::is_safe_argument(&reference.expr),
            syn::Expr::Paren(parenthesized) => Self::is_safe_argument(&parenthesized.expr),
            syn::Expr::Unary(unary) => {
                matches!(unary.op, syn::UnOp::Neg(_)) && Self::is_safe_argument(&unary.expr)
            }
            syn::Expr::Call(call) => Self::is_pointer_function_call(call),
            syn::Expr::MethodCall(call) => Self::is_pointer_method_call(call),
            _ => false,
        }
    }

    fn is_host_function_call(&self, expression: &syn::ExprUnsafe) -> bool {
        let index = &self.context.indexes[self.ordinal];
        if index.modules[self.module].tree != SourceTree::Test || !self.is_entry_point() {
            return false;
        }
        let [syn::Stmt::Expr(syn::Expr::Call(call), _)] = expression.block.stmts.as_slice() else {
            return false;
        };
        if !call.args.iter().all(Self::is_safe_argument) {
            return false;
        }
        let syn::Expr::Path(callee) = call.func.as_ref() else {
            return false;
        };
        let Resolution::Internal(targets) = index.resolve_path(self.module, &callee.path) else {
            return false;
        };
        targets.iter().any(|target| {
            let Some(identity) = target.item else {
                return false;
            };
            let item = &index.items[identity];
            let syn::Item::Fn(function) = &item.syntax else {
                return false;
            };
            item.module == 0 && Self::is_host_function(function)
        })
    }

    pub(crate) fn check_file(
        context: &RuleContext,
        ordinal: usize,
        file: &SourceFile,
    ) -> Vec<RuleFinding> {
        let Some(module) = context.indexes[ordinal]
            .modules
            .iter()
            .find(|module| module.module == file.module)
        else {
            return Vec::new();
        };
        let mut visitor = UnsafeVisitor {
            context,
            ordinal,
            module: module.id,
            file,
            findings: Vec::new(),
        };
        visitor.visit_file(&file.syntax);
        visitor.findings
    }
}

impl<'ast> Visit<'ast> for UnsafeVisitor<'_> {
    fn visit_macro(&mut self, value: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(value) {
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

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        let is_unsafe_attribute =
            matches!(&attribute.meta, syn::Meta::List(list) if list.path.is_ident("unsafe"));
        let is_host_entry = self.module == 0 && self.is_entry_point();
        if is_unsafe_attribute && !is_host_entry {
            self.check_span(attribute.span());
        }
        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if self.module == 0 && self.is_entry_point() && Self::is_host_function(item) {
            return;
        }
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_expr_unsafe(&mut self, expression: &'ast syn::ExprUnsafe) {
        if !self.is_host_function_call(expression) {
            self.check_span(expression.span());
        }
        syn::visit::visit_expr_unsafe(self, expression);
    }

    fn visit_signature(&mut self, signature: &'ast syn::Signature) {
        if matches!(signature.safety, syn::Safety::Unsafe(_)) {
            self.check_span(signature.span());
        }
        syn::visit::visit_signature(self, signature);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if item.unsafety.is_some() {
            self.check_span(item.span());
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        if item.unsafety.is_some() {
            self.check_span(item.span());
        }
        syn::visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if item.unsafety.is_some() {
            self.check_span(item.span());
        }
        syn::visit::visit_item_trait(self, item);
    }
}
