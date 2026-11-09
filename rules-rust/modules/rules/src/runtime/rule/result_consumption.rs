use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::{build_node_finding, format_module_path};
use crate::runtime::rule::TypedRuntimeVisitor;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) struct ResultConsumption;

struct ConsumptionVisitor<'source> {
    file: &'source SourceFile,
    allowed: bool,
    spans: Vec<proc_macro2::Span>,
}

fn expects_discard(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("expect") {
        return false;
    }
    let Ok(entries) = attribute.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    entries.iter().any(|entry| {
        matches!(entry, syn::Meta::Path(path)
        if path.segments.len() == 2 && path.segments[0].ident == "clippy"
        && path.segments[1].ident == "let_underscore_must_use")
    })
}

impl<'ast> Visit<'ast> for ConsumptionVisitor<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let parent = self.allowed;
        self.allowed = matches!(local.pat, syn::Pat::Wild(_));
        for attribute in &local.attrs {
            self.visit_attribute(attribute);
        }
        self.allowed = false;
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
        }
        self.allowed = parent;
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if !self.allowed && expects_discard(attribute) {
            self.spans.push(attribute.span());
        }
    }
}

impl Rule for ResultConsumption {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for file in &sources.files {
                if file.is_generated {
                    continue;
                }
                let mut visitor = ConsumptionVisitor {
                    file,
                    allowed: false,
                    spans: Vec::new(),
                };
                visitor.visit_file(&visitor.file.syntax);
                for span in visitor.spans {
                    findings.push(build_node_finding("ResultConsumption", &file.path, find_node_span(span),
                        format_module_path(&file.module.crate_name, &file.module.path),
                        "Scope let_underscore_must_use expectations to the intentional discard statement (error-handling.rs.md)".to_owned()));
                }
            }
        }
        findings.extend(TypedRuntimeVisitor::check_context(
            context,
            "ResultConsumption",
        ));
        findings
    }
}
