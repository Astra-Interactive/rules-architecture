use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::comment::rule::DocScopeVisitor;

pub(crate) struct DocCommentScope;

impl Rule for DocCommentScope {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| {
            let ordinal = context.indexes.iter().position(|index| index.crate_name == sources.crate_name && index.target == sources.target);
            let published = context.workspace.members.iter().find(|member| member.name == sources.crate_name).is_some_and(|member| member.manifest.root.find(&["package", "version"]).is_some());
            sources.files.iter().flat_map(move |file| ordinal.map(|ordinal| DocScopeVisitor::collect(context, ordinal, file, published)).unwrap_or_default().into_iter().map(|span| build_node_finding("DocCommentScope", &file.path, span, file.path.clone(), "docs add failure/safety contracts or external reasons; impl methods inherit trait docs and published panic contracts use headed sections".to_owned())))
        }).collect()
    }
}
