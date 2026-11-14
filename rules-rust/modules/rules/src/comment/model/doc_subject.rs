use crate::comment::model::DocOwner;

#[derive(Clone)]
pub(crate) enum DocSubject {
    Item(DocOwner),
    Member { owner: DocOwner, name: String },
}
