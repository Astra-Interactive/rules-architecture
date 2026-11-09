use renamed_provider::api::Contract as ReaderContract;
pub struct FileReader;
#[cfg(feature = "native")]
impl ReaderContract for FileReader {
    async fn load(&self) {}
}

#[cfg(not(feature = "native"))]
impl ReaderContract for FileReader {
    fn load(&self) {}
}
