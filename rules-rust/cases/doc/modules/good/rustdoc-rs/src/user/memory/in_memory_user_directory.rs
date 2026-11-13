use crate::user::model::{User, UserError, UserId};
use std::collections::HashMap;

pub struct InMemoryUserDirectory {
    users: HashMap<UserId, User>,
}

impl InMemoryUserDirectory {
    pub fn with_users(users: HashMap<UserId, User>) -> Self {
        Self { users }
    }

    /// # Errors
    ///
    /// Returns [`UserError::NotFound`] for a deleted user as well as for an unknown id.
    pub fn load_user(&self, user_id: UserId) -> Result<User, UserError> {
        self.users.get(&user_id).cloned().ok_or(UserError::NotFound)
    }
}
