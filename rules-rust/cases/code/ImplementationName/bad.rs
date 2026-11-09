mod api { trait Reader {} pub use Reader as Contract; }
mod model { struct NetworkReader; impl crate::api::Contract for NetworkReader {} }
