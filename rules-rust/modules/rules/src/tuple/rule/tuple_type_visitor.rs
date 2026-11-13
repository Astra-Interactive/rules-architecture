use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_expressions;
use proc_macro2::Span;
use syn::Fields;
use syn::ImplItem;
use syn::ItemImpl;
use syn::ItemMod;
use syn::ItemStruct;
use syn::TypeTuple;
use syn::Variant;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TupleTypeVisitor {
    file: String,
    module_path: String,
    findings: Vec<RuleFinding>,
}

impl TupleTypeVisitor {
    pub(crate) fn new(file: String, module_path: String) -> Self {
        Self {
            file,
            module_path,
            findings: Vec::new(),
        }
    }

    fn report(&mut self, span: Span, name: Option<String>, message: String) {
        let node = find_node_span(span);
        let subject = match name {
            Some(name) => format!("{}::{name}", self.module_path),
            None => self.module_path.clone(),
        };
        self.findings.push(build_node_finding(
            Self::RULE,
            &self.file,
            node,
            subject,
            message,
        ));
    }

    fn count_unnamed(fields: &Fields) -> usize {
        match fields {
            Fields::Unnamed(unnamed) => unnamed.unnamed.len(),
            _ => 0,
        }
    }

    fn visit_trait_impl(&mut self, item: &ItemImpl) {
        self.visit_generics(&item.generics);
        if let Some((path, _)) = &item.trait_ {
            self.visit_path(path);
        }
        self.visit_type(&item.self_ty);
        for impl_item in &item.items {
            match impl_item {
                ImplItem::Fn(function) => self.visit_block(&function.block),
                ImplItem::Macro(_) | ImplItem::Verbatim(_) => {}
                other => self.visit_impl_item(other),
            }
        }
    }

    fn visit_destructured_type(&mut self, pattern: &syn::Pat, typed: &syn::Type) {
        match (pattern, typed) {
            (syn::Pat::Tuple(bindings), syn::Type::Tuple(tuple)) => {
                let rest = bindings
                    .elems
                    .iter()
                    .position(|binding| matches!(binding, syn::Pat::Rest(_)));
                for (position, binding) in bindings.elems.iter().enumerate() {
                    if matches!(binding, syn::Pat::Rest(_)) {
                        continue;
                    }
                    let element = if rest.is_some_and(|rest| position > rest) {
                        tuple
                            .elems
                            .len()
                            .checked_sub(bindings.elems.len() - position)
                    } else {
                        Some(position)
                    };
                    if let Some(element) =
                        element.and_then(|position| tuple.elems.iter().nth(position))
                    {
                        self.visit_destructured_type(binding, element);
                    }
                }
            }
            (syn::Pat::Reference(binding), syn::Type::Reference(reference)) => {
                self.visit_destructured_type(&binding.pat, &reference.elem);
            }
            _ => self.visit_type(typed),
        }
    }

    pub(crate) fn collect_findings(mut self, syntax: &syn::File) -> Vec<RuleFinding> {
        self.visit_file(syntax);
        self.findings
    }

    const RULE: &str = "AnonymousTuple";
}

impl<'ast> Visit<'ast> for TupleTypeVisitor {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let syn::Pat::Type(pattern) = &local.pat {
            self.visit_pat(&pattern.pat);
            self.visit_destructured_type(&pattern.pat, &pattern.ty);
            if let Some(initializer) = &local.init {
                self.visit_expr(&initializer.expr);
                if let Some((_, diverging)) = &initializer.diverge {
                    self.visit_expr(diverging);
                }
            }
            return;
        }
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        for input in &closure.inputs {
            if let syn::Pat::Type(pattern) = input {
                self.visit_pat(&pattern.pat);
                self.visit_destructured_type(&pattern.pat, &pattern.ty);
            } else {
                self.visit_pat(input);
            }
        }
        self.visit_return_type(&closure.output);
        self.visit_expr(&closure.body);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_expressions(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_type_tuple(&mut self, tuple: &'ast TypeTuple) {
        if tuple.elems.len() >= 2 {
            let message = format!(
                "a tuple of {} elements as a type: use a struct with named fields (no-tuples.rs.md)",
                tuple.elems.len()
            );
            self.report(tuple.span(), None, message);
        }
        syn::visit::visit_type_tuple(self, tuple);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let count = Self::count_unnamed(&item.fields);
        if count >= 2 {
            let message =
                format!("a tuple struct of {count} fields: name its fields (no-tuples.rs.md)");
            self.report(item.span(), Some(item.ident.to_string()), message);
        }
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        let count = Self::count_unnamed(&variant.fields);
        if count >= 2 {
            let message =
                format!("a tuple variant of {count} fields: name its fields (no-tuples.rs.md)");
            self.report(variant.span(), Some(variant.ident.to_string()), message);
        }
        syn::visit::visit_variant(self, variant);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if item.trait_.is_some() {
            self.visit_trait_impl(item);
            return;
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let parent = self.module_path.clone();
        self.module_path = format!("{parent}::{}", item.ident.unraw());
        syn::visit::visit_item_mod(self, item);
        self.module_path = parent;
    }
}
