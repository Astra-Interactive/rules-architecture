mod model { struct Value; impl Value { fn read(&self) {} fn into_next(self) -> Self { self } } }
mod file { struct Row; impl Row { fn change(&mut self) {} } }
