use crate::source::model::SelectPart;
use crate::source::util::is_unlinked_item_macro;
use crate::source::util::parse_select_parts;
use crate::source::util::try_parse_macro_arguments;
use std::collections::BTreeSet;
use syn::visit::Visit;

pub(crate) struct NameAvailabilityVisitor {
    pub(crate) has_hidden_names: bool,
    pub(crate) has_linkable_target: bool,
    pub(crate) external_crates: BTreeSet<String>,
}

impl NameAvailabilityVisitor {
    fn is_known_macro(name: &str) -> bool {
        [
            "macro_rules",
            "assert",
            "assert_eq",
            "assert_ne",
            "debug_assert",
            "debug_assert_eq",
            "debug_assert_ne",
            "format",
            "format_args",
            "print",
            "println",
            "eprint",
            "eprintln",
            "vec",
            "matches",
            "panic",
            "unreachable",
            "todo",
            "unimplemented",
            "concat",
            "concat_bytes",
            "stringify",
            "env",
            "option_env",
            "file",
            "line",
            "column",
            "module_path",
        ]
        .contains(&name)
    }

    fn find_macro_name(invocation: &syn::Macro) -> String {
        invocation
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default()
    }

    fn visit_body_macro(&mut self, invocation: &syn::Macro) {
        let name = Self::find_macro_name(invocation);
        if name == "include" {
            self.has_hidden_names = true;
            return;
        }
        let is_select = !parse_select_parts(invocation).is_empty();
        if is_select || Self::is_known_macro(&name) {
            self.visit_macro(invocation);
            return;
        }
        let Some(arguments) = try_parse_macro_arguments(invocation) else {
            self.has_hidden_names = true;
            return;
        };
        for argument in &arguments {
            self.visit_expr(argument);
        }
    }
}

impl<'ast> Visit<'ast> for NameAvailabilityVisitor {
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if item.mac.path.is_ident("macro_rules") {
            self.has_hidden_names = true;
        }
        let is_unlinked =
            !self.has_linkable_target && is_unlinked_item_macro(item, &self.external_crates);
        if is_unlinked {
            return;
        }
        self.visit_macro(&item.mac);
    }
    fn visit_expr_macro(&mut self, expression: &'ast syn::ExprMacro) {
        self.visit_body_macro(&expression.mac);
    }
    fn visit_stmt_macro(&mut self, statement: &'ast syn::StmtMacro) {
        self.visit_body_macro(&statement.mac);
    }
    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        let select_parts = parse_select_parts(invocation);
        if !select_parts.is_empty() {
            for part in &select_parts {
                match part {
                    SelectPart::Pattern(pattern) => self.visit_pat(pattern),
                    SelectPart::Expression(expression) => self.visit_expr(expression),
                    SelectPart::BranchEnd => {}
                }
            }
            return;
        }
        if !Self::is_known_macro(&Self::find_macro_name(invocation)) {
            self.has_hidden_names = true;
        }
    }
}
