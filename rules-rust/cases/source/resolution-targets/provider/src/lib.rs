pub struct Digest;
pub struct Unused;
#[cfg(feature = "extra")]
pub struct FeatureRecord;
#[cfg(feature = "extra")]
pub type FeatureAlias = FeatureRecord;
#[cfg(not(test))]
pub struct ProductionRecord;
pub struct TestOnly;
pub struct ExternalRecord {
    pub revision: u64,
}
pub struct Model;
pub type Shared<T> = std::sync::Arc<T>;
pub type Pair<T> = (T, Model);

pub mod event {
    pub trait Registrar {
        fn register(&self, context: &framework_context::Context);
    }
}

pub mod model {
    mod exported;
    pub use exported::Exported;
}

pub struct BranchModel;
pub struct ConditionalOwner {
    #[cfg(unix)]
    pub value: BranchModel,
}
