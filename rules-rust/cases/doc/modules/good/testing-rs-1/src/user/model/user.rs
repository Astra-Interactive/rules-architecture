use crate::user::model::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct User {
    pub(crate) id: UserId,
    pub(crate) name: String,
}
