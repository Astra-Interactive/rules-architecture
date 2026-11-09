pub(crate) mod app {
    pub(crate) mod di {
        pub(crate) use core_module::CoreModule;
        mod core_module;
    }
    pub(crate) mod service {
        pub(crate) use task_scope::TaskScope;
        mod task_scope;
    }
}
pub(crate) mod feature_flag {
    pub(crate) mod di {
        pub(crate) use feature_flag_module::FeatureFlagModule;
        mod feature_flag_module;
    }
    pub(crate) mod service {
        pub(crate) use interval_feature_flag_reloader::IntervalFeatureFlagReloader;
        mod interval_feature_flag_reloader;
    }
}
pub(crate) mod order {
    pub(crate) mod component {
        pub(crate) use order_pricing::OrderPricing;
        mod order_pricing;
    }
    pub(crate) mod model {
        pub(crate) use money::Money;
        pub(crate) use order::Order;
        mod money;
        mod order;
    }
}
