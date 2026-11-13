use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::source::util::find_node_span;
use crate::source::util::format_tokens;
use crate::source::util::parse_macro_arguments;
use syn::ItemMod;
use syn::ItemUse;
use syn::UseTree;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct GlobImportVisitor {
    file: String,
    module_path: String,
    findings: Vec<RuleFinding>,
}

impl GlobImportVisitor {
    pub(crate) fn new(file: String, module_path: String) -> Self {
        Self {
            file,
            module_path,
            findings: Vec::new(),
        }
    }

    fn has_glob(tree: &UseTree) -> bool {
        match tree {
            UseTree::Glob(_) => true,
            UseTree::Path(path) => Self::has_glob(&path.tree),
            UseTree::Group(group) => group.items.iter().any(Self::has_glob),
            UseTree::Name(_) | UseTree::Rename(_) => false,
        }
    }

    pub(crate) fn collect_findings(mut self, syntax: &syn::File) -> Vec<RuleFinding> {
        self.visit_file(syntax);
        self.findings
    }

    const RULE: &str = "WildcardImport";
}

impl<'ast> Visit<'ast> for GlobImportVisitor {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let parent = self.module_path.clone();
        self.module_path = format!("{parent}::{}", item.ident.unraw());
        syn::visit::visit_item_mod(self, item);
        self.module_path = parent;
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        if !Self::has_glob(&item.tree) {
            return;
        }
        let written = format_tokens(&item.tree).replace(' ', "");
        let subject = format!("{} use {written}", self.module_path);
        let message = format!("`{written}` is a glob: import each item by name (imports.rs.md)");
        self.findings.push(build_node_finding(
            Self::RULE,
            &self.file,
            find_node_span(item.span()),
            subject,
            message,
        ));
    }
}
