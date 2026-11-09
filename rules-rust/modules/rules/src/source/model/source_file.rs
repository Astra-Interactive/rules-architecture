use crate::source::model::Comment;
use crate::source::model::ModuleId;

#[derive(Clone)]
pub(crate) struct SourceFile {
    pub(crate) path: String,
    pub(crate) module: ModuleId,
    pub(crate) is_generated: bool,
    pub(crate) syntax: syn::File,
    pub(crate) comments: Vec<Comment>,
}
