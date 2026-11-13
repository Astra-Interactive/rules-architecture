use crate::source::model::SelectPart;
use crate::source::util::parse_select_parts;
use syn::visit::Visit;

pub(crate) struct NameAvailabilityVisitor {
    pub(crate) has_hidden_names: bool,
}

impl<'ast> Visit<'ast> for NameAvailabilityVisitor {
    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        if item.mac.path.is_ident("macro_rules") {
            self.has_hidden_names = true;
        }
        self.visit_macro(&item.mac);
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
        let name = invocation
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        if ![
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
        .contains(&name.as_str())
        {
            self.has_hidden_names = true;
        }
    }
}
