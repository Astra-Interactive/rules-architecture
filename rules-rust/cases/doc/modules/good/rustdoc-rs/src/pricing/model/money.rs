#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money(i32);

impl Money {
    pub fn new(cents: i32) -> Self {
        Self(cents)
    }

    pub fn cents(&self) -> i32 {
        self.0
    }
}
