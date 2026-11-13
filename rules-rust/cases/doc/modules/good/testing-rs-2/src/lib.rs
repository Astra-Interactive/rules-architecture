pub(crate) mod order {
    pub(crate) mod api {
        pub(crate) use order_repository::OrderRepository;
        mod order_repository;
    }
    pub(crate) mod usecase {
        pub(crate) use create_order_use_case::CreateOrderUseCase;
        mod create_order_use_case;
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
}
