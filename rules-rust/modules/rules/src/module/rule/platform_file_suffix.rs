use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::source::model::NodeSpan;
use crate::source::util::find_node_span;
use crate::source::util::find_single_target_value;
use crate::source::util::format_tokens;
use crate::source::util::list_target_values;
use crate::source::util::parse_macro_arguments;
use std::collections::BTreeSet;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct PlatformFileSuffix {
    allowed: BTreeSet<NodeSpan>,
    spans: BTreeSet<NodeSpan>,
}

impl PlatformFileSuffix {
    pub(crate) fn new() -> Self {
        Self {
            allowed: BTreeSet::new(),
            spans: BTreeSet::new(),
        }
    }

    fn target_attributes(attributes: &[syn::Attribute]) -> Vec<&syn::Attribute> {
        attributes
            .iter()
            .filter(|attribute| !list_target_values(attribute).is_empty())
            .collect()
    }

    fn exported_names(tree: &syn::UseTree, owner: &str) -> BTreeSet<String> {
        match tree {
            syn::UseTree::Path(path) if path.ident == owner => Self::leaf_names(&path.tree),
            syn::UseTree::Group(group) => group
                .items
                .iter()
                .flat_map(|tree| Self::exported_names(tree, owner))
                .collect(),
            _ => BTreeSet::new(),
        }
    }

    fn leaf_names(tree: &syn::UseTree) -> BTreeSet<String> {
        match tree {
            syn::UseTree::Name(name) => BTreeSet::from([name.ident.unraw().to_string()]),
            syn::UseTree::Rename(name) => BTreeSet::from([name.rename.unraw().to_string()]),
            syn::UseTree::Path(path) => Self::leaf_names(&path.tree),
            syn::UseTree::Group(group) => group.items.iter().flat_map(Self::leaf_names).collect(),
            syn::UseTree::Glob(_) => BTreeSet::new(),
        }
    }

    fn check_exports(
        &mut self,
        items: &[syn::Item],
        name: &str,
        predicate: &syn::Attribute,
    ) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for item in items {
            let syn::Item::Use(import) = item else {
                continue;
            };
            if matches!(import.vis, syn::Visibility::Inherited) {
                continue;
            }
            let exports = Self::exported_names(&import.tree, name);
            if exports.is_empty() {
                continue;
            }
            names.extend(exports);
            let predicates = Self::target_attributes(&import.attrs);
            if predicates.len() == 1
                && format_tokens(&predicates[0].meta) == format_tokens(&predicate.meta)
            {
                self.allowed.insert(find_node_span(predicates[0].span()));
            } else {
                self.spans.insert(find_node_span(import.span()));
            }
        }
        names
    }

    fn check_scope(&mut self, items: &[syn::Item]) {
        let modules = items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Mod(module) if module.content.is_none() => Some(module),
                _ => None,
            })
            .collect::<Vec<_>>();
        for module in &modules {
            let targets = Self::target_attributes(&module.attrs);
            if targets.is_empty() {
                continue;
            }
            for attribute in &targets {
                self.allowed.insert(find_node_span(attribute.span()));
            }
            let value = find_single_target_value(&module.attrs);
            let name = module.ident.unraw().to_string();
            let Some(value) = value.filter(|_| targets.len() == 1) else {
                self.spans.insert(find_node_span(module.span()));
                continue;
            };
            let Some(stem) = name.strip_suffix(&format!("_{value}")) else {
                self.spans.insert(find_node_span(module.span()));
                continue;
            };
            let names = self.check_exports(items, &name, targets[0]);
            if names.is_empty() {
                self.spans.insert(find_node_span(module.span()));
            }
            for other in &modules {
                let other_name = other.ident.unraw().to_string();
                let Some(other_value) = find_single_target_value(&other.attrs) else {
                    continue;
                };
                if other_name.strip_suffix(&format!("_{other_value}")) != Some(stem) {
                    continue;
                }
                let other_names: BTreeSet<String> = items
                    .iter()
                    .filter_map(|item| match item {
                        syn::Item::Use(import)
                            if !matches!(import.vis, syn::Visibility::Inherited) =>
                        {
                            Some(import)
                        }
                        _ => None,
                    })
                    .flat_map(|import| Self::exported_names(&import.tree, &other_name))
                    .collect();
                if names != other_names {
                    self.spans.insert(find_node_span(module.span()));
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for PlatformFileSuffix {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_file(&mut self, file: &'ast syn::File) {
        self.check_scope(&file.items);
        syn::visit::visit_file(self, file);
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        if let Some((_, items)) = &module.content {
            self.check_scope(items);
        }
        syn::visit::visit_item_mod(self, module);
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !list_target_values(attribute).is_empty()
            && !self.allowed.contains(&find_node_span(attribute.span()))
        {
            self.spans.insert(find_node_span(attribute.span()));
        }
        syn::visit::visit_attribute(self, attribute);
    }
}

impl Rule for PlatformFileSuffix {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for file in context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .filter(|file| !file.is_generated)
        {
            let mut visitor = Self::new();
            visitor.visit_file(&file.syntax);
            findings.extend(visitor.spans.into_iter().map(|span| build_node_finding("PlatformFileSuffix", &file.path, span, file.path.clone(), "Put platform code in single-target suffixed file modules with matching conditional re-exports (arch-package.rs.md)".to_owned())));
        }
        findings
    }
}
