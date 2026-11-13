pub fn digest() -> digest_wire::Digest {
    const PAGE_SIZE: usize = 4096;
    let size = size_of::<u16>();
    assert!(size < PAGE_SIZE);
    drop(Box::new(size));
    digest_wire::Digest
}

#[cfg(not(feature = "extra"))]
pub fn consume_feature(_: digest_wire::FeatureRecord) {}

#[cfg(not(feature = "extra"))]
pub fn consume_feature_alias(_: digest_wire::FeatureAlias) {}

#[cfg(test)]
#[test]
fn consume_production() {
    let _: Option<digest_wire::ProductionRecord> = None;
}

#[cfg(test)]
#[path = "../test/lib.rs"]
mod test;

pub mod model {
    pub struct Snapshot {
        pub revision: u64,
    }
}

pub fn revise(mut record: digest_wire::ExternalRecord) -> digest_wire::ExternalRecord {
    record.revision = 1;
    record
}

pub struct Model;
pub fn shared(value: Model) -> digest_wire::Shared<Model> {
    std::sync::Arc::new(value)
}

pub mod usecase {
    pub fn register(registrar: &dyn digest_wire::event::Registrar) {
        let _ = registrar;
    }
}

pub fn exported() -> digest_wire::model::Exported {
    digest_wire::model::Exported
}

#[cfg(not(unix))]
pub fn conditional(value: digest_wire::ConditionalOwner) {
    let _ = value;
}
