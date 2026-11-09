use crate::check::api::Rule;
use crate::comment::rule::{
    DocCommentScope, ForbiddenComment, IntraDocLink, RustdocFiller, RustdocHtmlTag,
};
use std::sync::Arc;

pub(crate) struct CommentRuleModule {
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl CommentRuleModule {
    pub(crate) fn new() -> Self {
        let rules: Vec<Arc<dyn Rule>> = vec![
            Arc::new(ForbiddenComment),
            Arc::new(RustdocHtmlTag),
            Arc::new(RustdocFiller),
            Arc::new(DocCommentScope),
            Arc::new(IntraDocLink),
        ];
        let active_rule_ids = vec![
            "ForbiddenComment",
            "RustdocHtmlTag",
            "RustdocFiller",
            "DocCommentScope",
            "IntraDocLink",
        ];
        Self {
            rules: rules.into(),
            active_rule_ids,
        }
    }
}
