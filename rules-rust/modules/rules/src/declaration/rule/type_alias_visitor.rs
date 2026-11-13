use crate::check::model::RuleFinding;
use crate::check::util::build_child_module_path;
use crate::check::util::build_node_finding;
use crate::source::util::find_node_span;
use crate::source::util::visit_macro_body;
use proc_macro2::Span;
use syn::Generics;
use syn::Ident;
use syn::ImplItemType;
use syn::ItemMod;
use syn::ItemType;
use syn::TraitItemType;
use syn::Type;
use syn::TypeParamBound;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TypeAliasVisitor {
    file: String,
    module_path: String,
    findings: Vec<RuleFinding>,
}

impl TypeAliasVisitor {
    pub(crate) fn new(file: String, module_path: String) -> Self {
        Self {
            file,
            module_path,
            findings: Vec::new(),
        }
    }

    fn find_header_end(
        name: &Ident,
        generics: &Generics,
        last_bound: Option<&TypeParamBound>,
    ) -> Span {
        if let Some(bound) = last_bound {
            return bound.span();
        }
        generics
            .gt_token
            .as_ref()
            .map_or_else(|| name.span(), Spanned::span)
    }

    fn is_broken(
        type_span: Span,
        semicolon_span: Span,
        generics: &Generics,
        header_end: Span,
        aliased: &Type,
    ) -> bool {
        if generics.where_clause.is_none() {
            return type_span.start().line != semicolon_span.start().line;
        }
        let aliased_span = aliased.span();
        let is_header_broken = type_span.start().line != header_end.end().line;
        let is_type_broken = aliased_span.start().line != aliased_span.end().line;
        is_header_broken || is_type_broken
    }

    fn record_alias(
        &mut self,
        type_span: Span,
        semicolon_span: Span,
        generics: &Generics,
        header_end: Span,
        aliased: &Type,
        name: &Ident,
    ) {
        if !Self::is_broken(type_span, semicolon_span, generics, header_end, aliased) {
            return;
        }
        let name = name.unraw();
        let subject = format!("{} type {name}", self.module_path);
        let message = format!(
            "`type {name}` spans several lines: shorten the alias or name a generic argument as an alias of its own (clean-code.rs.md)"
        );
        self.findings.push(build_node_finding(
            Self::RULE,
            &self.file,
            find_node_span(type_span),
            subject,
            message,
        ));
    }

    pub(crate) fn collect_findings(mut self, syntax: &syn::File) -> Vec<RuleFinding> {
        self.visit_file(syntax);
        self.findings
    }

    const RULE: &str = "TypeAliasOnOneLine";
}

impl<'ast> Visit<'ast> for TypeAliasVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        visit_macro_body(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let child = build_child_module_path(&self.module_path, item);
        let parent = std::mem::replace(&mut self.module_path, child);
        syn::visit::visit_item_mod(self, item);
        self.module_path = parent;
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.record_alias(
            item.type_token.span(),
            item.semi_token.span(),
            &item.generics,
            Self::find_header_end(&item.ident, &item.generics, None),
            &item.ty,
            &item.ident,
        );
        syn::visit::visit_item_type(self, item);
    }

    fn visit_impl_item_type(&mut self, item: &'ast ImplItemType) {
        self.record_alias(
            item.type_token.span(),
            item.semi_token.span(),
            &item.generics,
            Self::find_header_end(&item.ident, &item.generics, None),
            &item.ty,
            &item.ident,
        );
        syn::visit::visit_impl_item_type(self, item);
    }

    fn visit_trait_item_type(&mut self, item: &'ast TraitItemType) {
        if let Some((_, aliased)) = &item.default {
            self.record_alias(
                item.type_token.span(),
                item.semi_token.span(),
                &item.generics,
                Self::find_header_end(&item.ident, &item.generics, item.bounds.last()),
                aliased,
                &item.ident,
            );
        }
        syn::visit::visit_trait_item_type(self, item);
    }
}
