use crate::source::model::NodeSpan;

pub(crate) struct HiddenMember<'a> {
    pub(crate) file: &'a str,
    pub(crate) name: String,
    pub(crate) span: NodeSpan,
    pub(crate) attributes: &'a [syn::Attribute],
    pub(crate) is_public: bool,
    pub(crate) is_trait_member: bool,
}
