mod model {
    struct Value;
    impl Value { fn create() -> Self { Self } }
}
fn create_value() -> Result<(), ()> { Ok(()) }
mod util { fn create_value() {} }
