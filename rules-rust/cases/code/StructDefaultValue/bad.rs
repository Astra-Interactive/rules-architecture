mod model { pub struct Input { pub value: u64 } }
fn build() -> model::Input {
    model::Input { value: 1, ..Default::default() }
}
