mod api {
    pub(crate) use contract::Contract;
    mod contract;
}
mod model {
    use crate::api::Contract;
    pub(crate) use value::Value;
    mod value;
}
