mod api { trait Reader {} }
mod model { struct InMemoryReader; }
mod memory { struct Wrong; impl crate::api::Reader for Wrong {} }
