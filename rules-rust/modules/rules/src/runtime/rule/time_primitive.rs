use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::util::node_predicate;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct TimePrimitive;

struct TimeVisitor<'context> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    spans: Vec<proc_macro2::Span>,
}

impl TimeVisitor<'_> {
    fn instant(&self, typed: &syn::Type) -> bool {
        let index = &self.context.indexes[self.ordinal];
        let cfg = node_predicate(
            index,
            self.module,
            &index.modules[self.module].file,
            find_node_span(typed.span()),
        );
        self.context
            .resolve_external_type_paths_under(self.ordinal, self.module, typed, &cfg)
            .iter()
            .any(|path| {
                matches!(
                    path.format_path().as_str(),
                    "std::time::Instant" | "tokio::time::Instant"
                )
            })
    }

    fn integer(&self, typed: &syn::Type) -> bool {
        self.context.resolve_type_syntax(self.ordinal, self.module, typed).iter().any(|resolved|
            matches!(&resolved.typed, syn::Type::Path(path) if path.path.get_ident().is_some_and(|name|
                ["u8","u16","u32","u64","u128","usize","i8","i16","i32","i64","i128","isize"].contains(&name.to_string().as_str()))))
    }
}

impl<'ast> Visit<'ast> for TimeVisitor<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let parent = self.module;
        if let Some(module) = self.context.indexes[self.ordinal]
            .modules
            .iter()
            .find(|module| {
                module.parent == Some(parent) && module.span == find_node_span(item.span())
            })
        {
            self.module = module.id;
        }
        syn::visit::visit_item_mod(self, item);
        self.module = parent;
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        if self.instant(&field.ty) {
            self.spans.push(field.span());
        }
        syn::visit::visit_field(self, field);
    }

    fn visit_return_type(&mut self, returned: &'ast syn::ReturnType) {
        if let syn::ReturnType::Type(_, typed) = returned
            && self.instant(typed)
        {
            self.spans.push(returned.span());
        }
        syn::visit::visit_return_type(self, returned);
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        if ["_MS", "_MILLIS", "_SECS", "_SECONDS", "_NANOS", "_MICROS"]
            .iter()
            .any(|suffix| item.ident.to_string().ends_with(suffix))
            && self.integer(&item.ty)
        {
            self.spans.push(item.span());
        }
        syn::visit::visit_item_const(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        if ["_MS", "_MILLIS", "_SECS", "_SECONDS", "_NANOS", "_MICROS"]
            .iter()
            .any(|suffix| item.ident.to_string().ends_with(suffix))
            && self.integer(&item.ty)
        {
            self.spans.push(item.span());
        }
        syn::visit::visit_impl_item_const(self, item);
    }
}

impl Rule for TimePrimitive {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                if file.is_generated {
                    continue;
                }
                let Some((ordinal, index)) =
                    context.indexes.iter().enumerate().find(|(_, index)| {
                        index.crate_name == file.module.crate_name
                            && index.target == file.module.target
                    })
                else {
                    continue;
                };
                let Some(module) = index
                    .modules
                    .iter()
                    .find(|module| module.module == file.module)
                else {
                    continue;
                };
                let mut visitor = TimeVisitor {
                    context,
                    ordinal,
                    module: module.id,
                    spans: Vec::new(),
                };
                visitor.visit_file(&file.syntax);
                for span in visitor.spans {
                    findings.push(build_node_finding("TimePrimitive", &file.path, find_node_span(span),
                format_module_path(&index.crate_name, &file.module.path), "Keep Instant process-local; represent time constants as Duration (time-api.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}
