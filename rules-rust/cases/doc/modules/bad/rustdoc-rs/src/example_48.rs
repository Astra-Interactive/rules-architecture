pub struct UserId(pub u64);

pub struct User;

pub enum UserError {
    NotFound,
}

pub struct UserDirectory;

impl UserDirectory {
    /// This function is intended for loading a user by their identifier.
    /// The function takes a user identifier and returns the user.
    ///
    /// `user_id` — the identifier of the user.
    ///
    /// Returns the user.
    pub fn load_user(&self, user_id: UserId) -> Result<User, UserError> {
        if user_id.0 == 0 {
            return Err(UserError::NotFound);
        }
        Ok(User)
    }
}
