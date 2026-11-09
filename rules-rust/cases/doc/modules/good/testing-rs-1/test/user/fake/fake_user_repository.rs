// test/user/fake/fake_user_repository.rs
use crate::user::api::UserRepository;
use crate::user::model::{SaveUserResult, User, UserId};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::collections::HashMap;

pub(crate) struct FakeUserRepository {
    users: Mutex<HashMap<UserId, User>>,
    saved: Mutex<Vec<User>>,
}

impl FakeUserRepository {
    pub(crate) fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
            saved: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn saved(&self) -> Vec<User> {
        self.saved.lock().clone()
    }
}

#[async_trait]
impl UserRepository for FakeUserRepository {
    async fn find_by_id(&self, id: UserId) -> Option<User> {
        self.users.lock().get(&id).cloned()
    }

    async fn save(&self, user: User) -> SaveUserResult {
        self.users.lock().insert(user.id, user.clone());
        self.saved.lock().push(user);
        SaveUserResult::Success
    }
}
