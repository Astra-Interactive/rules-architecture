use crate::test::user::fake::FakeUserRepository;
use crate::user::model::{SaveUserResult, User, UserId};
use crate::user::usecase::RegisterUserUseCase;
use std::sync::Arc;

#[tokio::test]
async fn given_new_user_when_registered_then_user_is_saved() {
    let fake_user_repository = Arc::new(FakeUserRepository::new());
    let register_user_use_case = RegisterUserUseCase::new(fake_user_repository.clone());
    let user = User {
        id: UserId(1),
        name: "Ada".to_owned(),
    };

    let result = register_user_use_case.invoke(user.clone()).await;

    assert_eq!(SaveUserResult::Success, result);
    assert_eq!(vec![user], fake_user_repository.saved());
}
