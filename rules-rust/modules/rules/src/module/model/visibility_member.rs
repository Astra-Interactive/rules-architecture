use crate::source::model::NodeSpan;

pub(crate) struct VisibilityMember<'a> {
    pub(crate) file: &'a str,
    pub(crate) span: NodeSpan,
    pub(crate) name: String,
    pub(crate) visibility: &'a syn::Visibility,
}
