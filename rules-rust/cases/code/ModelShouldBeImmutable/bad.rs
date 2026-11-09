mod model { struct Value; impl Value { fn change(&mut self) {} } }
mod util { impl crate::model::Value { fn reset(&mut self) {} } }
