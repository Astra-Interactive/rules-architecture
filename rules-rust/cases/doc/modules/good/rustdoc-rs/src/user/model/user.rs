use crate::user::model::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
}
