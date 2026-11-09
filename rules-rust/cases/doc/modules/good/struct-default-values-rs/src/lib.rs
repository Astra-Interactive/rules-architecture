pub(crate) mod prediction {
    pub(crate) mod model {
        pub(crate) use prediction_config::PredictionConfig;
        pub(crate) use prediction_input::PredictionInput;
        mod prediction_config;
        mod prediction_input;
    }
    pub(crate) mod util {
        pub(crate) use input_build::build_prediction_inputs;
        mod input_build;
    }
}
