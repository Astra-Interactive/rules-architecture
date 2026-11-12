use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::find_kind_name;
use crate::source::model::SourceFile;
use crate::source::util::{find_node_span, parse_macro_arguments};
use syn::spanned::Spanned;
use syn::visit::Visit;

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
        let index = &self.context.indexes[self.ordinal];
        let is_host_entry = self.module == 0
            && self
                .context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                == Some(crate::config::model::Role::EntryPoint);
        if is_unsafe_attribute && !is_host_entry {
            self.check_span(attribute.span());
        }
        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_expr_unsafe(&mut self, expression: &'ast syn::ExprUnsafe) {
        self.check_span(expression.span());
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
