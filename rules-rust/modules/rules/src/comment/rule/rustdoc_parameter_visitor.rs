use crate::comment::util::joined_doc;
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

    fn split_named_line(line: &str) -> Option<NamedLine<'_>> {
        let line = line.trim_start();
        let item = line.strip_prefix("- ").or_else(|| line.strip_prefix("* "));
        let line = item.unwrap_or(line).trim_start();
        if let Some(rest) = line.strip_prefix('`') {
            let (name, description) = rest.split_once('`')?;
            return Some(NamedLine { name, description });
        }
        item?;
        let length = line
            .find(|character: char| !character.is_alphanumeric() && character != '_')
            .unwrap_or(line.len());
        let (name, description) = line.split_at(length);
        Some(NamedLine { name, description })
    }

    fn repeats(line: &str, parameters: &[String]) -> bool {
        let Some(NamedLine { name, description }) = Self::split_named_line(line) else {
            return false;
        };
        if !parameters.iter().any(|parameter| parameter == name) {
            return false;
        }
        let description = description.trim_start();
        let Some(description) = Self::DESCRIPTION_SEPARATORS
            .iter()
            .find_map(|separator| description.strip_prefix(*separator))
        else {
            return false;
        };
        let mut name = Self::words(name);
        let mut description = Self::words(description);
        name.sort();
        description.sort();
        !name.is_empty() && name == description
    }

    fn lists_parameter(text: &str, name: &str) -> bool {
        text.lines().any(|line| {
            let line = line.trim_start();
            let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) else {
                return false;
            };
            item.trim_start()
                .strip_prefix('`')
                .and_then(|rest| rest.strip_prefix(name))
                .is_some_and(|rest| rest.starts_with('`'))
        })
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
        let reported = self.spans.len();
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
        let typed = signature
            .inputs
            .iter()
            .filter(|input| matches!(input, syn::FnArg::Typed(_)))
            .count();
        let is_unreported = self.spans.len() == reported;
        let is_fully_typed = parameters.len() > 1 && parameters.len() == typed;
        if !is_unreported || !is_fully_typed {
            return;
        }
        let Some((span, text)) = joined_doc(attributes) else {
            return;
        };
        let lists_every_parameter = parameters
            .iter()
            .all(|parameter| Self::lists_parameter(&text, parameter));
        if lists_every_parameter {
            self.spans.push(span);
        }
    }

    const DESCRIPTION_SEPARATORS: [char; 4] = ['—', '–', '-', ':'];
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

struct NamedLine<'text> {
    name: &'text str,
    description: &'text str,
}
