use crate::user::model::{SaveUserResult, User, UserId};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Option<User>;
    async fn save(&self, user: User) -> SaveUserResult;
}
