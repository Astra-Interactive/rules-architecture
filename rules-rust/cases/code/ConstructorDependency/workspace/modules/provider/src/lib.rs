pub mod api { pub trait Repository {} }
pub mod file {
    pub struct DiskRepository;
    impl crate::api::Repository for DiskRepository {}
    impl DiskRepository { pub fn new() -> Self { Self } }
}
pub use file::DiskRepository as Exported;
