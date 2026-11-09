mod first {
    pub(crate) mod model {
        use crate::second::model::Second;
        pub(crate) use first::First;
        mod first;
    }
}
mod second {
    pub(crate) mod model {
        use crate::first::model::First;
        pub(crate) use second::Second;
        mod second;
    }
}
