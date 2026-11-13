use crate::source::util::format_tokens;
use crate::source::util::parse_macro_arguments;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct LintExpectationVisitor {
    is_module: bool,
    is_container: bool,
    allows_print: bool,
    findings: Vec<proc_macro2::Span>,
}

impl LintExpectationVisitor {
    pub(crate) fn new(allows_print: bool) -> Self {
        Self {
            is_module: false,
            is_container: false,
            allows_print,
            findings: Vec::new(),
        }
    }

    fn is_prescribed_lint(&self, name: &str) -> bool {
        match name {
            "clippy::unwrap_used"
            | "clippy::expect_used"
            | "clippy::let_underscore_must_use"
            | "clippy::disallowed_methods"
            | "clippy::disallowed_types" => true,
            "clippy::print_stdout" | "clippy::print_stderr" => self.allows_print,
            _ => false,
        }
    }

    fn has_blank_reason(
        arguments: &syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>,
    ) -> bool {
        arguments.iter().any(|argument| {
            let syn::Meta::NameValue(pair) = argument else {
                return false;
            };
            let syn::Expr::Lit(literal) = &pair.value else {
                return false;
            };
            let syn::Lit::Str(reason) = &literal.lit else {
                return false;
            };
            pair.path.is_ident("reason") && reason.value().trim().is_empty()
        })
    }

    fn expects_disallowed_methods(attribute: &syn::Attribute) -> bool {
        let syn::Meta::List(list) = &attribute.meta else {
            return false;
        };
        let parser = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
        list.path.is_ident("expect")
            && parser.parse2(list.tokens.clone()).is_ok_and(|arguments| {
                arguments.iter().any(|argument| {
                    matches!(argument, syn::Meta::Path(path)
                        if format_tokens(path).replace(' ', "") == "clippy::disallowed_methods")
                })
            })
    }

    fn writes_console(block: &syn::Block) -> bool {
        let text = format_tokens(block).replace(' ', "");
        text.contains("io::stdout(") || text.contains("io::stderr(")
    }

    fn check_console_expectation(&mut self, attributes: &[syn::Attribute], block: &syn::Block) {
        if self.allows_print || !Self::writes_console(block) {
            return;
        }
        let spans = attributes
            .iter()
            .filter(|attribute| Self::expects_disallowed_methods(attribute))
            .map(Spanned::span);
        self.findings.extend(spans);
    }

    fn violates_scope(&self, meta: &syn::Meta, is_inner: bool, is_conditional: bool) -> bool {
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
                .any(|argument| self.violates_scope(argument, is_inner, true));
        }
        let is_expect = list.path.is_ident("expect");
        let is_allow = (is_inner || is_conditional) && list.path.is_ident("allow");
        if !is_expect && !is_allow {
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
        if is_expect && Self::has_blank_reason(&arguments) {
            return true;
        }
        let names: Vec<String> = lints
            .iter()
            .map(|path| format_tokens(path).replace(' ', ""))
            .collect();
        let covers_functions = self.is_container
            && names
                .iter()
                .any(|name| Self::EXPRESSION_LINTS.contains(&name.as_str()));
        is_inner
            || self.is_module
            || is_allow
            || covers_functions
            || names.iter().any(|name| !self.is_prescribed_lint(name))
    }

    fn visit_container_attributes(&mut self, attributes: &[syn::Attribute]) {
        self.is_container = true;
        for attribute in attributes {
            self.visit_attribute(attribute);
        }
        self.is_container = false;
    }

    pub(crate) fn collect_findings(mut self, syntax: &syn::File) -> Vec<proc_macro2::Span> {
        self.visit_file(syntax);
        self.findings
    }

    const EXPRESSION_LINTS: [&'static str; 6] = [
        "clippy::unwrap_used",
        "clippy::expect_used",
        "clippy::let_underscore_must_use",
        "clippy::disallowed_methods",
        "clippy::print_stdout",
        "clippy::print_stderr",
    ];
}

impl<'ast> Visit<'ast> for LintExpectationVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        let is_inner = matches!(attribute.style, syn::AttrStyle::Inner(_));
        if self.violates_scope(&attribute.meta, is_inner, false) {
            self.findings.push(attribute.span());
        }
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.check_console_expectation(&item.attrs, &item.block);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.check_console_expectation(&item.attrs, &item.block);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        self.visit_container_attributes(&item.attrs);
        for child in &item.items {
            self.visit_impl_item(child);
        }
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.visit_container_attributes(&item.attrs);
        for child in &item.items {
            self.visit_trait_item(child);
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
