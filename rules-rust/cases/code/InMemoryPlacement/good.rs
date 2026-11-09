mod api { trait Reader {} pub use Reader as Contract; }
mod memory { struct InMemoryReader; impl crate::api::Contract for InMemoryReader {} }
