use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::declaration::rule::CallbackVisitor;
use crate::source::model::SourceTree;
use crate::source::util::find_node_span;

pub(crate) struct CallbackDeclaration;

impl CallbackDeclaration {
    fn field_findings(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut findings = Vec::new();
        let syn::Item::Struct(structure) = &item.syntax else {
            return Vec::new();
        };
        let production = crate::source::model::CfgPredicate::Not(Box::new(
            crate::source::model::CfgPredicate::Atom {
                key: "test".to_owned(),
                value: None,
            },
        ));
        if index.modules[item.module].tree != SourceTree::Src
            || !item.cfg.and(&production).is_satisfiable()
        {
            return Vec::new();
        }
        for field in &structure.fields {
            let cfg = item
                .cfg
                .and(&crate::source::model::CfgPredicate::from_attributes(
                    &field.attrs,
                ));
            let span = find_node_span(syn::spanned::Spanned::span(field));
            if crate::declaration::util::callback_field_type(
                context,
                ordinal,
                item.module,
                &field.ty,
                &cfg,
                &structure.generics,
                0,
            ) {
                findings.push(build_node_finding(
                    "CallbackDeclaration",
                    &item.file,
                    span,
                    item.file.clone(),
                    "return async results or streams instead of storing callbacks".to_owned(),
                ));
            }
        }

        findings
    }
}

impl Rule for CallbackDeclaration {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for file in context.crates.iter().flat_map(|sources| &sources.files) {
            let is_test_tree = context
                .indexes
                .iter()
                .flat_map(|index| &index.modules)
                .any(|module| module.module == file.module && module.tree != SourceTree::Src);
            if is_test_tree {
                continue;
            }
            let Some((ordinal, index)) = context.indexes.iter().enumerate().find(|(_, index)| {
                index.crate_name == file.module.crate_name && index.target == file.module.target
            }) else {
                continue;
            };
            let Some(module) = index
                .modules
                .iter()
                .find(|module| module.module == file.module)
            else {
                continue;
            };
            let spans = CallbackVisitor::new(context, ordinal, module.id).collect_spans(file);
            findings.extend(spans.into_iter().map(|span| build_node_finding("CallbackDeclaration", &file.path, find_node_span(span), file.path.clone(), "Return async results or streams instead of declaring callbacks (no-callbacks.rs.md)".to_owned())));
        }
        let candidates = context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .items
                    .iter()
                    .flat_map(move |item| Self::field_findings(context, ordinal, item))
            });
        for finding in candidates {
            if !findings
                .iter()
                .any(|existing| existing.anchor == finding.anchor)
            {
                findings.push(finding);
            }
        }
        findings
    }
}
