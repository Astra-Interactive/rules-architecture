use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::source::model::NodeSpan;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::source::util::parse_macro_arguments;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct LocalItem {
    spans: Vec<NodeSpan>,
}

impl LocalItem {
    pub(crate) fn new() -> Self {
        Self { spans: Vec::new() }
    }

    fn check_file(file: &SourceFile) -> Vec<RuleFinding> {
        let mut visitor = Self::new();
        visitor.visit_file(&file.syntax);
        visitor.spans.into_iter().map(|span| build_node_finding("LocalItem", &file.path, span, file.path.clone(), "Declare types, impls and modules outside executable bodies (arch-package.rs.md)".to_owned())).collect()
    }
}

impl<'ast> Visit<'ast> for LocalItem {
    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        for argument in &parse_macro_arguments(item) {
            self.visit_expr(argument);
        }
    }

    fn visit_stmt(&mut self, statement: &'ast syn::Stmt) {
        if let syn::Stmt::Item(item) = statement
            && matches!(
                item,
                syn::Item::Struct(_)
                    | syn::Item::Enum(_)
                    | syn::Item::Union(_)
                    | syn::Item::Trait(_)
                    | syn::Item::Impl(_)
                    | syn::Item::Mod(_)
            )
        {
            self.spans.push(find_node_span(item.span()));
        }
        syn::visit::visit_stmt(self, statement);
    }
}

impl Rule for LocalItem {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .filter(|file| !file.is_generated)
            .flat_map(Self::check_file)
            .collect()
    }
}
