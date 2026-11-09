pub(crate) mod sample {
    pub(crate) mod model {
        pub(crate) use result::Outcome;
        pub(crate) use payload::Payload;
        mod result;
        mod payload;
    }
}
