mod model {
    struct Value;
    impl Value { fn create() -> Self { Self } }
}
fn create_value() -> Result<model::Value, ()> { Ok(model::Value) }
mod util { fn create_value() -> crate::model::Value { crate::model::Value } }
