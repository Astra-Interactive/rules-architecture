use crate::dictionary::model::{DictionaryError, DictionaryId};
use crate::user::model::User;

pub trait DictionaryRepository {
    /// # Errors
    ///
    /// Returns [`DictionaryError::NotFound`] when no dictionary has this id.
    fn delete_dictionary(&self, id: DictionaryId, user: &User) -> Result<(), DictionaryError>;
}
