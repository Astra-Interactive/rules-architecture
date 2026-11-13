pub mod dictionary {
    pub mod api {
        pub use dictionary_repository::DictionaryRepository;
        mod dictionary_repository;
    }
    pub mod model {
        pub use dictionary_error::DictionaryError;
        pub use dictionary_id::DictionaryId;
        mod dictionary_error;
        mod dictionary_id;
    }
}
pub mod order {
    pub mod memory {
        pub use order_book::OrderBook;
        mod order_book;
    }
    pub mod model {
        pub use order::Order;
        mod order;
    }
}
pub mod pricing {
    pub mod model {
        pub use money::Money;
        mod money;
    }
    pub(crate) mod util {
        mod money_ext;
    }
}
pub mod user {
    pub mod memory {
        pub use in_memory_user_directory::InMemoryUserDirectory;
        mod in_memory_user_directory;
    }
    pub mod model {
        pub use user::User;
        pub use user_error::UserError;
        pub use user_id::UserId;
        mod user;
        mod user_error;
        mod user_id;
    }
}
