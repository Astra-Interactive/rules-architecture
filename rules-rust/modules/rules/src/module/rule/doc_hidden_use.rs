use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::DocHiddenVisitor;

pub(crate) struct DocHiddenUse;

impl Rule for DocHiddenUse {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            let published_library = context
                .workspace
                .members
                .iter()
                .find(|member| member.name == sources.crate_name)
                .is_some_and(|member| {
                    member.is_published(&context.workspace) && member.has_rust_library()
                });
            if !published_library {
                continue;
            }
            for file in sources.files.iter().filter(|file| !file.is_generated) {
                let Some(index) = context.indexes.iter().find(|index| {
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
                findings.extend(DocHiddenVisitor::collect(file, module.cfg.clone()));
            }
        }
        for (ordinal, index) in context
            .indexes
            .iter()
            .enumerate()
            .filter(|(ordinal, _)| context.can_prove_workspace_only(*ordinal))
        {
            for (identity, item) in index
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| RuleContext::is_public_item(item))
            {
                let attrs = match &item.syntax {
                    syn::Item::Struct(item) => &item.attrs,
                    syn::Item::Enum(item) => &item.attrs,
                    syn::Item::Trait(item) => &item.attrs,
                    syn::Item::Type(item) => &item.attrs,
                    syn::Item::Fn(item) => &item.attrs,
                    _ => continue,
                };
                if attrs.iter().any(DocHiddenVisitor::hides)
                    && !context.has_external_name_user(ordinal, identity)
                {
                    findings.push(build_node_finding("DocHiddenUse",&item.file,item.span,item.name.clone(),"reserve doc(hidden) public visibility for actual consumers in another crate".to_owned()));
                }
            }
        }
        findings
    }
}
