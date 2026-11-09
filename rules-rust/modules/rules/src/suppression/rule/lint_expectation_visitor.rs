use crate::source::util::format_tokens;
use crate::source::util::parse_macro_arguments;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct LintExpectationVisitor {
    is_module: bool,
    findings: Vec<proc_macro2::Span>,
}

impl LintExpectationVisitor {
    pub(crate) fn new() -> Self {
        Self {
            is_module: false,
            findings: Vec::new(),
        }
    }

    fn violates_scope(meta: &syn::Meta, is_inner: bool, is_module: bool) -> bool {
        let syn::Meta::List(list) = meta else {
            return false;
        };
        let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
        let Ok(arguments) = parser.parse2(list.tokens.clone()) else {
            return false;
        };
        if list.path.is_ident("cfg_attr") {
            if matches!(arguments.first(), Some(syn::Meta::Path(path)) if path.is_ident("klibs_rules"))
            {
                return false;
            }
            return arguments
                .iter()
                .skip(1)
                .any(|argument| Self::violates_scope(argument, is_inner, is_module));
        }
        let is_expect = list.path.is_ident("expect");
        let is_inner_allow = is_inner && list.path.is_ident("allow");
        if !is_expect && !is_inner_allow {
            return false;
        }
        let lints: Vec<&syn::Path> = arguments
            .iter()
            .filter_map(|argument| match argument {
                syn::Meta::Path(path) => Some(path),
                _ => None,
            })
            .collect();
        if lints.is_empty() {
            return false;
        }
        is_inner
            || is_module
            || lints.iter().any(|path| {
                let name = format_tokens(path).replace(' ', "");
                ![
                    "clippy::unwrap_used",
                    "clippy::expect_used",
                    "clippy::let_underscore_must_use",
                    "clippy::disallowed_methods",
                    "clippy::disallowed_types",
                    "clippy::print_stdout",
                ]
                .contains(&name.as_str())
            })
    }

    pub(crate) fn collect_findings(mut self, syntax: &syn::File) -> Vec<proc_macro2::Span> {
        self.visit_file(syntax);
        self.findings
    }
}

impl<'ast> Visit<'ast> for LintExpectationVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        let is_inner = matches!(attribute.style, syn::AttrStyle::Inner(_));
        if Self::violates_scope(&attribute.meta, is_inner, self.is_module) {
            self.findings.push(attribute.span());
        }
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.is_module = true;
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        self.is_module = false;
        if let Some((_, children)) = &item.content {
            for child in children {
                self.visit_item(child);
            }
        }
    }
}
