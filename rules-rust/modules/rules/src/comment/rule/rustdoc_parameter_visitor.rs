use crate::source::util::find_node_span;
use crate::syntax::util::to_snake_case;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct RustdocParameterVisitor {
    spans: Vec<crate::source::model::NodeSpan>,
}

impl RustdocParameterVisitor {
    pub(crate) fn collect(syntax: &syn::File) -> Vec<crate::source::model::NodeSpan> {
        let mut visitor = Self { spans: Vec::new() };
        visitor.visit_file(syntax);
        visitor.spans
    }

    fn words(value: &str) -> Vec<String> {
        to_snake_case(value)
            .split(|character: char| !character.is_alphanumeric())
            .filter(|word| !word.is_empty() && !["a", "an", "the"].contains(word))
            .map(str::to_owned)
            .collect()
    }

    fn repeats(line: &str, parameters: &[String]) -> bool {
        let Some(rest) = line.trim_start().strip_prefix('`') else {
            return false;
        };
        let Some((name, description)) = rest.split_once('`') else {
            return false;
        };
        if !parameters.iter().any(|parameter| parameter == name) {
            return false;
        }
        let description = description.trim_start();
        let Some(description) = description
            .strip_prefix('—')
            .or_else(|| description.strip_prefix('-'))
        else {
            return false;
        };
        let mut name = Self::words(name);
        let mut description = Self::words(description);
        name.sort();
        description.sort();
        !name.is_empty() && name == description
    }

    fn inspect(&mut self, signature: &syn::Signature, attributes: &[syn::Attribute]) {
        let parameters: Vec<_> = signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(parameter) => match parameter.pat.as_ref() {
                    syn::Pat::Ident(binding) => Some(binding.ident.unraw().to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        for attribute in attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("doc"))
        {
            let syn::Meta::NameValue(value) = &attribute.meta else {
                continue;
            };
            let syn::Expr::Lit(value) = &value.value else {
                continue;
            };
            let syn::Lit::Str(text) = &value.lit else {
                continue;
            };
            if text
                .value()
                .lines()
                .any(|line| Self::repeats(line, &parameters))
            {
                self.spans.push(find_node_span(attribute.span()));
            }
        }
    }
}

impl<'ast> Visit<'ast> for RustdocParameterVisitor {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.inspect(&function.sig, &function.attrs);
        syn::visit::visit_item_fn(self, function);
    }
    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.inspect(&function.sig, &function.attrs);
        syn::visit::visit_impl_item_fn(self, function);
    }
    fn visit_trait_item_fn(&mut self, function: &'ast syn::TraitItemFn) {
        self.inspect(&function.sig, &function.attrs);
        syn::visit::visit_trait_item_fn(self, function);
    }
}
