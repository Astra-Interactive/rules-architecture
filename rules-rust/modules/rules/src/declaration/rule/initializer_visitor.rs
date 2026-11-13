use crate::check::model::RuleFinding;
use crate::check::util::build_child_module_path;
use crate::check::util::build_node_finding;
use crate::source::util::find_node_span;
use crate::source::util::format_tokens;
use crate::source::util::visit_macro_body;
use proc_macro2::Span;
use syn::Expr;
use syn::ImplItemConst;
use syn::ItemConst;
use syn::ItemMod;
use syn::ItemStatic;
use syn::Local;
use syn::TraitItemConst;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct InitializerVisitor {
    file: String,
    module_path: String,
    findings: Vec<RuleFinding>,
}

impl InitializerVisitor {
    pub(crate) fn new(file: String, module_path: String) -> Self {
        Self {
            file,
            module_path,
            findings: Vec::new(),
        }
    }

    fn record_declaration(
        &mut self,
        keyword_span: Span,
        equals_span: Span,
        initializer: &Expr,
        declaration: &str,
    ) {
        if initializer.span().start().line <= equals_span.start().line {
            return;
        }
        let subject = format!("{} {declaration}", self.module_path);
        let message = format!(
            "the initializer of `{declaration}` starts on a line after its `=`: shorten the declaration or name a part of it on its own (clean-code.rs.md)"
        );
        self.findings.push(build_node_finding(
            Self::RULE,
            &self.file,
            find_node_span(keyword_span),
            subject,
            message,
        ));
    }

    pub(crate) fn collect_findings(mut self, syntax: &syn::File) -> Vec<RuleFinding> {
        self.visit_file(syntax);
        self.findings
    }

    const RULE: &str = "InitializerOnDeclarationLine";
}

impl<'ast> Visit<'ast> for InitializerVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        visit_macro_body(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let child = build_child_module_path(&self.module_path, item);
        let parent = std::mem::replace(&mut self.module_path, child);
        syn::visit::visit_item_mod(self, item);
        self.module_path = parent;
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if let Some(init) = &local.init {
            let pattern = format_tokens(&local.pat);
            self.record_declaration(
                local.let_token.span(),
                init.eq_token.span(),
                &init.expr,
                &format!("let {pattern}"),
            );
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.record_declaration(
            item.const_token.span(),
            item.eq_token.span(),
            &item.expr,
            &format!("const {}", item.ident.unraw()),
        );
        syn::visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.record_declaration(
            item.static_token.span(),
            item.eq_token.span(),
            &item.expr,
            &format!("static {}", item.ident.unraw()),
        );
        syn::visit::visit_item_static(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.record_declaration(
            item.const_token.span(),
            item.eq_token.span(),
            &item.expr,
            &format!("const {}", item.ident.unraw()),
        );
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast TraitItemConst) {
        if let Some((eq_token, initializer)) = &item.default {
            self.record_declaration(
                item.const_token.span(),
                eq_token.span(),
                initializer,
                &format!("const {}", item.ident.unraw()),
            );
        }
        syn::visit::visit_trait_item_const(self, item);
    }
}
