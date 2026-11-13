mod api { trait Reader {} pub use Reader as Contract; }
mod model { struct DefaultReader; impl crate::api::Contract for DefaultReader {} }
mod file { struct FileReader; impl crate::api::Contract for FileReader {} }
