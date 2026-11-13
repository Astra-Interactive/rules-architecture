pub(crate) mod order {
    pub(crate) mod api {
        pub(crate) use order_repository::OrderRepository;
        mod order_repository;
    }
    pub(crate) mod model {
        pub(crate) use create_order_command::CreateOrderCommand;
        pub(crate) use create_order_result::CreateOrderResult;
        pub(crate) use customer_id::CustomerId;
        mod create_order_command;
        mod create_order_result;
        mod customer_id;
    }
    pub(crate) mod usecase {
        pub(crate) use create_order_use_case::CreateOrderUseCase;
        mod create_order_use_case;
    }
}
pub(crate) mod payment {
    pub(crate) mod api {
        pub(crate) use payment_gateway::PaymentGateway;
        mod payment_gateway;
    }
}
pub(crate) mod user {
    pub(crate) mod api {
        pub(crate) use user_repository::UserRepository;
        mod user_repository;
    }
    pub(crate) mod model {
        pub(crate) use save_user_result::SaveUserResult;
        pub(crate) use user::User;
        pub(crate) use user_id::UserId;
        mod save_user_result;
        mod user;
        mod user_id;
    }
    pub(crate) mod usecase {
        pub(crate) use register_user_use_case::RegisterUserUseCase;
        mod register_user_use_case;
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod order {
        pub(crate) mod fake {
            pub(crate) use fake_order_repository::FakeOrderRepository;
            mod fake_order_repository;
        }
        mod usecase {
            mod create_order_use_case_test;
        }
    }
    mod payment {
        pub(crate) mod fake {
            pub(crate) use fake_payment_gateway::FakePaymentGateway;
            mod fake_payment_gateway;
        }
    }
    mod user {
        pub(crate) mod fake {
            pub(crate) use fake_user_repository::FakeUserRepository;
            mod fake_user_repository;
        }
        mod usecase {
            mod register_user_use_case_test;
        }
    }
}
