pub(crate) mod order {
    pub(crate) mod command {
        pub(crate) use order_report_command::run_order_report;
        mod order_report_command;
    }
    pub(crate) mod model {
        pub(crate) use order::Order;
        pub(crate) use order_error::OrderError;
        pub(crate) use order_id::OrderId;
        mod order;
        mod order_error;
        mod order_id;
    }
    pub(crate) mod util {
        pub(crate) use order_load::load_order;
        mod order_load;
    }
}
