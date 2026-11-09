use crate::user::api::UserRepository;
use crate::user::model::{SaveUserResult, User};
use std::sync::Arc;

pub(crate) struct RegisterUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl RegisterUserUseCase {
    pub(crate) fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub(crate) async fn invoke(&self, user: User) -> SaveUserResult {
        if self.user_repository.find_by_id(user.id).await.is_some() {
            return SaveUserResult::AlreadyRegistered;
        }
        self.user_repository.save(user).await
    }
}
